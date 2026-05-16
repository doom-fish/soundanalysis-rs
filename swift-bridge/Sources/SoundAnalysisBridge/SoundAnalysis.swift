// SoundAnalysis Bridge
//
// @_cdecl wrappers around Apple's SoundAnalysis framework so Rust can
// classify sounds in audio files using Apple's built-in classifier
// (`SNClassifierIdentifierVersion1` — 200+ everyday sounds).

import Foundation
import SoundAnalysis
import AVFoundation
import CoreMedia

// ----- Status codes (match src/ffi/mod.rs::status) -----

private let SA_OK: Int32 = 0
private let SA_INVALID_ARGUMENT: Int32 = -1
private let SA_AUDIO_LOAD_FAILED: Int32 = -2
private let SA_REQUEST_CREATE_FAILED: Int32 = -3
private let SA_ANALYSIS_FAILED: Int32 = -4
private let SA_UNKNOWN: Int32 = -99

// MARK: - String helpers

private func ffiString(_ s: String) -> UnsafeMutablePointer<CChar>? {
    return strdup(s)
}

@_cdecl("sa_string_free")
public func sa_string_free(_ s: UnsafeMutablePointer<CChar>?) {
    guard let s = s else { return }
    free(s)
}

// MARK: - Layout-compatible structs (mirror Rust ffi/mod.rs)

public struct SAClassificationRaw {
    /// NUL-terminated category identifier, e.g. "speech", "music",
    /// "applause", "dog_bark". Caller frees via `sa_classification_results_free`.
    public var identifier: UnsafeMutablePointer<CChar>?
    public var confidence: Double
}

public struct SAClassificationResultRaw {
    /// Time range start (seconds since file start).
    public var time_start: Double
    /// Time range duration (seconds).
    public var time_duration: Double
    /// Pointer to a flat C array of `SAClassificationRaw` (length =
    /// `classification_count`). Caller frees via the parent results
    /// `sa_classification_results_free`.
    public var classifications: UnsafeMutablePointer<SAClassificationRaw>?
    public var classification_count: Int
}

// MARK: - Built-in classifier metadata

@_cdecl("sa_known_classifications")
public func sa_known_classifications(
    _ outArray: UnsafeMutablePointer<UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?>,
    _ outCount: UnsafeMutablePointer<Int>
) -> Int32 {
    do {
        let request = try SNClassifySoundRequest(classifierIdentifier: .version1)
        let labels = request.knownClassifications
        if labels.isEmpty {
            outArray.pointee = nil
            outCount.pointee = 0
            return SA_OK
        }
        let buffer = UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>.allocate(
            capacity: labels.count
        )
        for (i, l) in labels.enumerated() {
            buffer.advanced(by: i).initialize(to: ffiString(l))
        }
        outArray.pointee = buffer
        outCount.pointee = labels.count
        return SA_OK
    } catch {
        return SA_REQUEST_CREATE_FAILED
    }
}

@_cdecl("sa_known_classifications_free")
public func sa_known_classifications_free(
    _ array: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ count: Int
) {
    guard let array = array else { return }
    for i in 0..<count {
        if let p = array.advanced(by: i).pointee { free(p) }
    }
    array.deallocate()
}

// MARK: - Result observer

private final class CollectingObserver: NSObject, SNResultsObserving {
    var results: [SAClassificationResultRaw] = []
    var error: Error?

    func request(_ request: SNRequest, didProduce result: SNResult) {
        guard let cls = result as? SNClassificationResult else { return }
        let timeStart = CMTimeGetSeconds(cls.timeRange.start)
        let timeDuration = CMTimeGetSeconds(cls.timeRange.duration)
        let count = cls.classifications.count
        let buffer = UnsafeMutablePointer<SAClassificationRaw>.allocate(capacity: count)
        for (i, c) in cls.classifications.enumerated() {
            buffer.advanced(by: i).initialize(to: SAClassificationRaw(
                identifier: ffiString(c.identifier),
                confidence: c.confidence
            ))
        }
        results.append(SAClassificationResultRaw(
            time_start: timeStart,
            time_duration: timeDuration,
            classifications: buffer,
            classification_count: count
        ))
    }

    func request(_ request: SNRequest, didFailWithError error: Error) {
        self.error = error
    }

    func requestDidComplete(_ request: SNRequest) {}
}

// MARK: - File classification

/// Synchronously analyze the audio file at `audioPath` using Apple's
/// built-in classifier (`SNClassifierIdentifierVersion1`).
///
/// Returns a flat array of `SAClassificationResultRaw` (one per analysis
/// window). Rust frees via `sa_classification_results_free`.
@_cdecl("sa_classify_file")
public func sa_classify_file(
    _ audioPath: UnsafePointer<CChar>,
    _ outArray: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ outCount: UnsafeMutablePointer<Int>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let path = String(cString: audioPath)
    let url = URL(fileURLWithPath: path)

    let analyzer: SNAudioFileAnalyzer
    do {
        analyzer = try SNAudioFileAnalyzer(url: url)
    } catch {
        outErrorMessage?.pointee = ffiString("audio file load failed: \(error.localizedDescription)")
        return SA_AUDIO_LOAD_FAILED
    }

    let request: SNClassifySoundRequest
    do {
        request = try SNClassifySoundRequest(classifierIdentifier: .version1)
    } catch {
        outErrorMessage?.pointee = ffiString("classifier init failed: \(error.localizedDescription)")
        return SA_REQUEST_CREATE_FAILED
    }

    let observer = CollectingObserver()
    do {
        try analyzer.add(request, withObserver: observer)
    } catch {
        outErrorMessage?.pointee = ffiString("add request failed: \(error.localizedDescription)")
        return SA_ANALYSIS_FAILED
    }

    // SNAudioFileAnalyzer.analyze() is synchronous; it returns when the
    // entire file has been processed (or an observer received an error).
    analyzer.analyze()

    if let err = observer.error {
        outErrorMessage?.pointee = ffiString("analysis failed: \(err.localizedDescription)")
        return SA_ANALYSIS_FAILED
    }

    if observer.results.isEmpty {
        outArray.pointee = nil
        outCount.pointee = 0
        return SA_OK
    }
    let buffer = UnsafeMutablePointer<SAClassificationResultRaw>.allocate(
        capacity: observer.results.count
    )
    for (i, r) in observer.results.enumerated() {
        buffer.advanced(by: i).initialize(to: r)
    }
    outArray.pointee = UnsafeMutableRawPointer(buffer)
    outCount.pointee = observer.results.count
    return SA_OK
}

@_cdecl("sa_classification_results_free")
public func sa_classification_results_free(_ array: UnsafeMutableRawPointer?, _ count: Int) {
    guard let array = array else { return }
    let typed = array.assumingMemoryBound(to: SAClassificationResultRaw.self)
    for i in 0..<count {
        let r = typed.advanced(by: i).pointee
        if let inner = r.classifications {
            for j in 0..<r.classification_count {
                if let id = inner.advanced(by: j).pointee.identifier { free(id) }
            }
            inner.deallocate()
        }
    }
    typed.deallocate()
}

// MARK: - Live mic streaming (v0.2)

/// Callback shape:
///   user_info, time_start, time_duration, classifications_ptr, classification_count
public typealias SAStreamCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Double, Double,
    UnsafeMutableRawPointer?,
    Int
) -> Void

@available(macOS 10.15, *)
private final class SAStreamObserver: NSObject, SNResultsObserving {
    let callback: SAStreamCallback
    let userInfo: UnsafeMutableRawPointer?

    init(callback: SAStreamCallback, userInfo: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.userInfo = userInfo
    }

    func request(_ request: SNRequest, didProduce result: SNResult) {
        guard let classification = result as? SNClassificationResult else { return }
        let range = classification.timeRange
        let timeStart = CMTimeGetSeconds(range.start)
        let timeDur = CMTimeGetSeconds(range.duration)

        let classifications = classification.classifications
        let n = classifications.count
        if n == 0 {
            callback(userInfo, timeStart, timeDur, nil, 0)
            return
        }
        let buf = UnsafeMutablePointer<SAClassificationRaw>.allocate(capacity: n)
        for (i, c) in classifications.enumerated() {
            buf.advanced(by: i).initialize(to: SAClassificationRaw(
                identifier: strdup(c.identifier),
                confidence: c.confidence
            ))
        }
        callback(userInfo, timeStart, timeDur, UnsafeMutableRawPointer(buf), n)
        for i in 0..<n {
            if let s = buf[i].identifier { free(s) }
        }
        buf.deallocate()
    }
}

private final class SAStreamSession {
    let engine = AVAudioEngine()
    let analyzer: SNAudioStreamAnalyzer
    let observer: SAStreamObserver
    var framePos: AVAudioFramePosition = 0

    init(observer: SAStreamObserver) throws {
        let input = engine.inputNode
        let format = input.outputFormat(forBus: 0)
        analyzer = SNAudioStreamAnalyzer(format: format)
        self.observer = observer
        let request = try SNClassifySoundRequest(classifierIdentifier: .version1)
        try analyzer.add(request, withObserver: observer)
        input.installTap(onBus: 0, bufferSize: 8192, format: format) { [weak self] buf, _ in
            guard let self = self else { return }
            self.analyzer.analyze(buf, atAudioFramePosition: self.framePos)
            self.framePos += AVAudioFramePosition(buf.frameLength)
        }
    }

    func start() throws { try engine.start() }
    func stop() {
        engine.stop()
        engine.inputNode.removeTap(onBus: 0)
        analyzer.completeAnalysis()
    }
}

private var streamSessions: [UnsafeMutableRawPointer: SAStreamSession] = [:]
private let streamSessionsLock = NSLock()

@_cdecl("sa_stream_start")
public func sa_stream_start(
    _ callback: SAStreamCallback,
    _ user_info: UnsafeMutableRawPointer?,
    _ out_err: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    if #unavailable(macOS 10.15) {
        out_err?.pointee = ffiString("requires macOS 10.15+")
        return nil
    }
    do {
        let observer = SAStreamObserver(callback: callback, userInfo: user_info)
        let session = try SAStreamSession(observer: observer)
        try session.start()
        let key = Unmanaged.passRetained(session).toOpaque()
        streamSessionsLock.lock()
        streamSessions[key] = session
        streamSessionsLock.unlock()
        return key
    } catch {
        out_err?.pointee = ffiString("stream start failed: \(error.localizedDescription)")
        return nil
    }
}

@_cdecl("sa_stream_stop")
public func sa_stream_stop(_ handle: UnsafeMutableRawPointer?) {
    guard let handle = handle else { return }
    streamSessionsLock.lock()
    let session = streamSessions.removeValue(forKey: handle)
    streamSessionsLock.unlock()
    session?.stop()
    Unmanaged<SAStreamSession>.fromOpaque(handle).release()
}
