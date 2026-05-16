import AVFoundation
import CoreML
import CoreMedia
import Foundation
import SoundAnalysis

// MARK: - Live microphone convenience surface

public typealias SAStreamCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Double,
    Double,
    UnsafeMutableRawPointer?,
    Int
) -> Void

@available(macOS 10.15, *)
private final class SAStreamObserver: NSObject, SNResultsObserving {
    let callback: SAStreamCallback
    let userInfo: UnsafeMutableRawPointer?

    init(callback: @escaping SAStreamCallback, userInfo: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.userInfo = userInfo
    }

    func request(_ request: SNRequest, didProduce result: SNResult) {
        guard let classification = result as? SNClassificationResult else { return }
        let buffer = classificationsRaw(classification.classifications)
        callback(
            userInfo,
            CMTimeGetSeconds(classification.timeRange.start),
            CMTimeGetSeconds(classification.timeRange.duration),
            UnsafeMutableRawPointer(buffer),
            classification.classifications.count
        )
        freeClassificationsRaw(buffer, count: classification.classifications.count)
    }
}

private final class SAStreamSession {
    let engine = AVAudioEngine()
    let analyzer: SNAudioStreamAnalyzer
    let observer: SAStreamObserver
    var framePosition: AVAudioFramePosition = 0

    init(observer: SAStreamObserver) throws {
        let input = engine.inputNode
        let format = input.outputFormat(forBus: 0)
        analyzer = SNAudioStreamAnalyzer(format: format)
        self.observer = observer
        let request = try builtInRequest(classifier: SA_CLASSIFIER_IDENTIFIER_VERSION1)
        try analyzer.add(request, withObserver: observer)
        input.installTap(onBus: 0, bufferSize: 8192, format: format) { [weak self] buffer, _ in
            guard let self else { return }
            analyzer.analyze(buffer, atAudioFramePosition: framePosition)
            framePosition += AVAudioFramePosition(buffer.frameLength)
        }
    }

    func start() throws {
        try engine.start()
    }

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
    _ callback: @escaping SAStreamCallback,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    if #unavailable(macOS 10.15) {
        outErrorMessage?.pointee = ffiString("requires macOS 10.15+")
        return nil
    }
    do {
        let observer = SAStreamObserver(callback: callback, userInfo: userInfo)
        let session = try SAStreamSession(observer: observer)
        try session.start()
        let key = Unmanaged.passRetained(session).toOpaque()
        streamSessionsLock.lock()
        streamSessions[key] = session
        streamSessionsLock.unlock()
        return key
    } catch {
        outErrorMessage?.pointee = ffiString("stream start failed: \(error.localizedDescription)")
        return nil
    }
}

@_cdecl("sa_stream_stop")
public func sa_stream_stop(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    streamSessionsLock.lock()
    let session = streamSessions.removeValue(forKey: handle)
    streamSessionsLock.unlock()
    session?.stop()
    Unmanaged<SAStreamSession>.fromOpaque(handle).release()
}
