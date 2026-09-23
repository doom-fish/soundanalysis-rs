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

public typealias SAContextCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

private final class SAContextRetention {
    let context: UnsafeMutableRawPointer?
    private let release: SAContextCallback?

    init(context: UnsafeMutableRawPointer?, retain: SAContextCallback?, release: SAContextCallback?) {
        self.context = context
        self.release = release
        if let context, let retain {
            retain(context)
        }
    }

    deinit {
        if let context, let release {
            release(context)
        }
    }
}

@available(macOS 10.15, *)
private final class SAStreamObserver: NSObject, SNResultsObserving {
    let callback: SAStreamCallback
    let errorCallback: SAObserverErrorCallback
    let completeCallback: SAObserverCompleteCallback
    let retention: SAContextRetention

    init(
        callback: @escaping SAStreamCallback,
        errorCallback: @escaping SAObserverErrorCallback,
        completeCallback: @escaping SAObserverCompleteCallback,
        retention: SAContextRetention
    ) {
        self.callback = callback
        self.errorCallback = errorCallback
        self.completeCallback = completeCallback
        self.retention = retention
    }

    func request(_ request: SNRequest, didProduce result: SNResult) {
        guard let classification = result as? SNClassificationResult else { return }
        let buffer = classificationsRaw(classification.classifications)
        callback(
            retention.context,
            CMTimeGetSeconds(classification.timeRange.start),
            CMTimeGetSeconds(classification.timeRange.duration),
            UnsafeMutableRawPointer(buffer),
            classification.classifications.count
        )
        freeClassificationsRaw(buffer, count: classification.classifications.count)
    }

    func request(_ request: SNRequest, didFailWithError error: Error) {
        errorCallback(
            retention.context,
            SA_ANALYSIS_FAILED,
            ffiString("analysis failed: \(error.localizedDescription)")
        )
    }

    func requestDidComplete(_ request: SNRequest) {
        completeCallback(retention.context)
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
        guard format.sampleRate > 0, format.channelCount > 0 else {
            throw bridgeError("no audio input device is available", status: SA_ANALYSIS_FAILED)
        }
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
        analyzer.removeAllRequests()
    }
}

private var streamSessions: [UnsafeMutableRawPointer: SAStreamSession] = [:]
private let streamSessionsLock = NSLock()

@_cdecl("sa_stream_start")
public func sa_stream_start(
    _ callback: @escaping SAStreamCallback,
    _ errorCallback: @escaping SAObserverErrorCallback,
    _ completeCallback: @escaping SAObserverCompleteCallback,
    _ userInfo: UnsafeMutableRawPointer?,
    _ retain: SAContextCallback?,
    _ release: SAContextCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    if #unavailable(macOS 10.15) {
        outErrorMessage?.pointee = ffiString("requires macOS 10.15+")
        return nil
    }
    do {
        let observer = SAStreamObserver(
            callback: callback,
            errorCallback: errorCallback,
            completeCallback: completeCallback,
            retention: SAContextRetention(context: userInfo, retain: retain, release: release)
        )
        let session = try SAStreamSession(observer: observer)
        do {
            try session.start()
        } catch {
            session.stop()
            throw error
        }
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

@_cdecl("sa_live_microphone_authorized")
public func sa_live_microphone_authorized() -> Bool {
    AVCaptureDevice.authorizationStatus(for: .audio) == .authorized
}
