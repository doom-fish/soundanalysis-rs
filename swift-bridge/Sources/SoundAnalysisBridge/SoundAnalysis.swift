import AVFoundation
import CoreML
import CoreMedia
import Foundation
import SoundAnalysis

// MARK: - Known classifications

@_cdecl("sa_known_classifications")
public func sa_known_classifications(
    _ outArray: UnsafeMutablePointer<UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?>,
    _ outCount: UnsafeMutablePointer<Int>
) -> Int32 {
    do {
        let request = try builtInRequest(classifier: SA_CLASSIFIER_IDENTIFIER_VERSION1)
        copyStrings(request.knownClassifications, outArray: outArray, outCount: outCount)
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
    guard let array else { return }
    for index in 0..<count {
        if let string = array.advanced(by: index).pointee {
            free(string)
        }
    }
    array.deallocate()
}

// MARK: - Request surface

@_cdecl("sa_request_create_classifier")
public func sa_request_create_classifier(
    _ classifier: Int32,
    _ outRequest: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outRequest.pointee = nil
    do {
        let request = try builtInRequest(classifier: classifier)
        outRequest.pointee = saRetain(request)
        return SA_OK
    } catch {
        return fail(outErrorMessage, defaultStatus: SA_REQUEST_CREATE_FAILED, error: error)
    }
}

@_cdecl("sa_request_create_model")
public func sa_request_create_model(
    _ modelPath: UnsafePointer<CChar>,
    _ outRequest: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outRequest.pointee = nil
    let modelURL = URL(fileURLWithPath: String(cString: modelPath))
    do {
        let model = try MLModel(contentsOf: modelURL)
        let request = try SNClassifySoundRequest(mlModel: model)
        outRequest.pointee = saRetain(request)
        return SA_OK
    } catch {
        return fail(outErrorMessage, defaultStatus: SA_REQUEST_CREATE_FAILED, error: error)
    }
}

@_cdecl("sa_request_retain")
public func sa_request_retain(_ request: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let request else { return nil }
    let object: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    return saRetain(object)
}

@_cdecl("sa_request_release")
public func sa_request_release(_ request: UnsafeMutableRawPointer?) {
    guard let request else { return }
    saRelease(request, as: SNClassifySoundRequest.self)
}

@_cdecl("sa_request_get_overlap_factor")
public func sa_request_get_overlap_factor(_ request: UnsafeMutableRawPointer?) -> Double {
    guard let request else { return 0 }
    let object: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    return object.overlapFactor
}

@_cdecl("sa_request_set_overlap_factor")
public func sa_request_set_overlap_factor(
    _ request: UnsafeMutableRawPointer?,
    _ overlapFactor: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let request else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null SNClassifySoundRequest")
    }
    guard overlapFactor >= 0, overlapFactor < 1 else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "overlapFactor must be in 0.0..<1.0")
    }
    let object: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    object.overlapFactor = overlapFactor
    return SA_OK
}

@_cdecl("sa_request_get_window_duration")
public func sa_request_get_window_duration(_ request: UnsafeMutableRawPointer?) -> Double {
    guard let request else { return 0 }
    let object: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    return CMTimeGetSeconds(object.windowDuration)
}

@_cdecl("sa_request_set_window_duration")
public func sa_request_set_window_duration(
    _ request: UnsafeMutableRawPointer?,
    _ seconds: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let request else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null SNClassifySoundRequest")
    }
    guard seconds.isFinite, seconds > 0 else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "windowDuration must be finite and > 0")
    }
    let object: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    object.windowDuration = secondsToCMTime(seconds)
    return SA_OK
}

@_cdecl("sa_request_get_window_duration_constraint")
public func sa_request_get_window_duration_constraint(
    _ request: UnsafeMutableRawPointer?,
    _ outConstraint: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let request else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null SNClassifySoundRequest")
    }
    let object: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    return fillConstraintRaw(object.windowDurationConstraint, outRaw: outConstraint)
}

@_cdecl("sa_request_known_classifications_for_request")
public func sa_request_known_classifications_for_request(
    _ request: UnsafeMutableRawPointer?,
    _ outArray: UnsafeMutablePointer<UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?>,
    _ outCount: UnsafeMutablePointer<Int>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let request else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null SNClassifySoundRequest")
    }
    let object: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    copyStrings(object.knownClassifications, outArray: outArray, outCount: outCount)
    return SA_OK
}

// MARK: - TimeDurationConstraint helpers

@_cdecl("sa_time_duration_constraint_create_enumerated")
public func sa_time_duration_constraint_create_enumerated(
    _ durations: UnsafePointer<Double>?,
    _ count: Int,
    _ outConstraint: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard count > 0, let durations else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "enumerated constraint requires at least one duration")
    }
    let values = UnsafeBufferPointer(start: durations, count: count).map(secondsToCMTime)
    return fillConstraintRaw(.enumeratedDurations(values), outRaw: outConstraint)
}

@_cdecl("sa_time_duration_constraint_create_range")
public func sa_time_duration_constraint_create_range(
    _ startSeconds: Double,
    _ durationSeconds: Double,
    _ outConstraint: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard durationSeconds >= 0 else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "duration range cannot be negative")
    }
    let range = CMTimeRange(start: secondsToCMTime(startSeconds), duration: secondsToCMTime(durationSeconds))
    return fillConstraintRaw(.durationRange(range), outRaw: outConstraint)
}

// MARK: - Analyzer wrappers

@_cdecl("sa_audio_file_analyzer_create")
public func sa_audio_file_analyzer_create(
    _ audioPath: UnsafePointer<CChar>,
    _ outAnalyzer: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outAnalyzer.pointee = nil
    let url = URL(fileURLWithPath: String(cString: audioPath))
    do {
        let box = try SAFileAnalyzerBox(url: url)
        outAnalyzer.pointee = saRetain(box)
        return SA_OK
    } catch {
        return fail(outErrorMessage, defaultStatus: SA_AUDIO_LOAD_FAILED, error: error)
    }
}

@_cdecl("sa_audio_file_analyzer_release")
public func sa_audio_file_analyzer_release(_ analyzer: UnsafeMutableRawPointer?) {
    guard let analyzer else { return }
    saRelease(analyzer, as: SAFileAnalyzerBox.self)
}

@_cdecl("sa_audio_file_analyzer_add_request")
public func sa_audio_file_analyzer_add_request(
    _ analyzer: UnsafeMutableRawPointer?,
    _ request: UnsafeMutableRawPointer?,
    _ userInfo: UnsafeMutableRawPointer?,
    _ resultCallback: @escaping SAObserverResultCallback,
    _ errorCallback: @escaping SAObserverErrorCallback,
    _ completeCallback: @escaping SAObserverCompleteCallback,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let analyzer, let request else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null analyzer or request")
    }

    let box: SAFileAnalyzerBox = saBorrow(analyzer, as: SAFileAnalyzerBox.self)
    let requestObject: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    let key = UInt(bitPattern: request)
    let observer = SAObserverBox(
        userInfo: userInfo,
        resultCallback: resultCallback,
        errorCallback: errorCallback,
        completeCallback: completeCallback,
        recordError: { [weak box] (error: Error) in
            if box?.latestAnalysisError == nil {
                box?.latestAnalysisError = error
            }
        }
    )

    do {
        try box.analyzer.add(requestObject, withObserver: observer)
        box.observers[key] = observer
        return SA_OK
    } catch {
        return fail(outErrorMessage, defaultStatus: SA_ANALYSIS_FAILED, error: error)
    }
}

@_cdecl("sa_audio_file_analyzer_remove_request")
public func sa_audio_file_analyzer_remove_request(
    _ analyzer: UnsafeMutableRawPointer?,
    _ request: UnsafeMutableRawPointer?
) {
    guard let analyzer, let request else { return }
    let box: SAFileAnalyzerBox = saBorrow(analyzer, as: SAFileAnalyzerBox.self)
    let requestObject: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    box.analyzer.remove(requestObject)
    box.observers.removeValue(forKey: UInt(bitPattern: request))
}

@_cdecl("sa_audio_file_analyzer_remove_all_requests")
public func sa_audio_file_analyzer_remove_all_requests(_ analyzer: UnsafeMutableRawPointer?) {
    guard let analyzer else { return }
    let box: SAFileAnalyzerBox = saBorrow(analyzer, as: SAFileAnalyzerBox.self)
    box.analyzer.removeAllRequests()
    box.observers.removeAll()
}

@_cdecl("sa_audio_file_analyzer_analyze")
public func sa_audio_file_analyzer_analyze(
    _ analyzer: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let analyzer else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null analyzer")
    }
    let box: SAFileAnalyzerBox = saBorrow(analyzer, as: SAFileAnalyzerBox.self)
    box.latestAnalysisError = nil
    box.analyzer.analyze()
    if let error = box.latestAnalysisError {
        return fail(outErrorMessage, status: SA_ANALYSIS_FAILED, message: "analysis failed: \(error.localizedDescription)")
    }
    return SA_OK
}

@_cdecl("sa_audio_file_analyzer_analyze_with_completion")
public func sa_audio_file_analyzer_analyze_with_completion(
    _ analyzer: UnsafeMutableRawPointer?,
    _ outDidReachEndOfFile: UnsafeMutablePointer<Bool>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let analyzer else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null analyzer")
    }
    let box: SAFileAnalyzerBox = saBorrow(analyzer, as: SAFileAnalyzerBox.self)
    let semaphore = DispatchSemaphore(value: 0)
    var didReachEndOfFile = false
    box.latestAnalysisError = nil
    box.analyzer.analyze { reachedEndOfFile in
        didReachEndOfFile = reachedEndOfFile
        semaphore.signal()
    }
    semaphore.wait()
    outDidReachEndOfFile.pointee = didReachEndOfFile
    if let error = box.latestAnalysisError {
        return fail(outErrorMessage, status: SA_ANALYSIS_FAILED, message: "analysis failed: \(error.localizedDescription)")
    }
    return SA_OK
}

@_cdecl("sa_audio_file_analyzer_cancel_analysis")
public func sa_audio_file_analyzer_cancel_analysis(_ analyzer: UnsafeMutableRawPointer?) {
    guard let analyzer else { return }
    let box: SAFileAnalyzerBox = saBorrow(analyzer, as: SAFileAnalyzerBox.self)
    box.analyzer.cancelAnalysis()
}

@_cdecl("sa_audio_stream_analyzer_create")
public func sa_audio_stream_analyzer_create(
    _ format: UnsafeRawPointer?,
    _ outAnalyzer: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outAnalyzer.pointee = nil
    guard let format else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null stream format")
    }
    let raw = format.assumingMemoryBound(to: SAStreamFormatRaw.self).pointee
    guard let avFormat = makeAVAudioFormat(from: raw) else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "invalid stream format")
    }
    let box = SAStreamAnalyzerBox(format: avFormat)
    outAnalyzer.pointee = saRetain(box)
    return SA_OK
}

@_cdecl("sa_audio_stream_analyzer_release")
public func sa_audio_stream_analyzer_release(_ analyzer: UnsafeMutableRawPointer?) {
    guard let analyzer else { return }
    saRelease(analyzer, as: SAStreamAnalyzerBox.self)
}

@_cdecl("sa_audio_stream_analyzer_add_request")
public func sa_audio_stream_analyzer_add_request(
    _ analyzer: UnsafeMutableRawPointer?,
    _ request: UnsafeMutableRawPointer?,
    _ userInfo: UnsafeMutableRawPointer?,
    _ resultCallback: @escaping SAObserverResultCallback,
    _ errorCallback: @escaping SAObserverErrorCallback,
    _ completeCallback: @escaping SAObserverCompleteCallback,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let analyzer, let request else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null analyzer or request")
    }

    let box: SAStreamAnalyzerBox = saBorrow(analyzer, as: SAStreamAnalyzerBox.self)
    let requestObject: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    let key = UInt(bitPattern: request)
    let observer = SAObserverBox(
        userInfo: userInfo,
        resultCallback: resultCallback,
        errorCallback: errorCallback,
        completeCallback: completeCallback,
        recordError: { [weak box] (error: Error) in
            if box?.latestAnalysisError == nil {
                box?.latestAnalysisError = error
            }
        }
    )

    do {
        try box.analyzer.add(requestObject, withObserver: observer)
        box.observers[key] = observer
        return SA_OK
    } catch {
        return fail(outErrorMessage, defaultStatus: SA_ANALYSIS_FAILED, error: error)
    }
}

@_cdecl("sa_audio_stream_analyzer_remove_request")
public func sa_audio_stream_analyzer_remove_request(
    _ analyzer: UnsafeMutableRawPointer?,
    _ request: UnsafeMutableRawPointer?
) {
    guard let analyzer, let request else { return }
    let box: SAStreamAnalyzerBox = saBorrow(analyzer, as: SAStreamAnalyzerBox.self)
    let requestObject: SNClassifySoundRequest = saBorrow(request, as: SNClassifySoundRequest.self)
    box.analyzer.remove(requestObject)
    box.observers.removeValue(forKey: UInt(bitPattern: request))
}

@_cdecl("sa_audio_stream_analyzer_remove_all_requests")
public func sa_audio_stream_analyzer_remove_all_requests(_ analyzer: UnsafeMutableRawPointer?) {
    guard let analyzer else { return }
    let box: SAStreamAnalyzerBox = saBorrow(analyzer, as: SAStreamAnalyzerBox.self)
    box.analyzer.removeAllRequests()
    box.observers.removeAll()
}

@_cdecl("sa_audio_stream_analyzer_analyze_audio_buffer")
public func sa_audio_stream_analyzer_analyze_audio_buffer(
    _ analyzer: UnsafeMutableRawPointer?,
    _ buffer: UnsafeRawPointer?,
    _ audioFramePosition: Int64,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let analyzer, let buffer else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null analyzer or buffer")
    }
    guard audioFramePosition >= 0 else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "audioFramePosition must be non-negative")
    }

    let box: SAStreamAnalyzerBox = saBorrow(analyzer, as: SAStreamAnalyzerBox.self)
    let raw = buffer.assumingMemoryBound(to: SAStreamBufferRaw.self).pointee
    guard let audioBuffer = makePCMBuffer(from: raw, format: box.format) else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "PCM buffer layout does not match analyzer format")
    }

    box.latestAnalysisError = nil
    box.analyzer.analyze(audioBuffer, atAudioFramePosition: AVAudioFramePosition(audioFramePosition))
    if let error = box.latestAnalysisError {
        return fail(outErrorMessage, status: SA_ANALYSIS_FAILED, message: "analysis failed: \(error.localizedDescription)")
    }
    return SA_OK
}

@_cdecl("sa_audio_stream_analyzer_complete_analysis")
public func sa_audio_stream_analyzer_complete_analysis(_ analyzer: UnsafeMutableRawPointer?) {
    guard let analyzer else { return }
    let box: SAStreamAnalyzerBox = saBorrow(analyzer, as: SAStreamAnalyzerBox.self)
    box.analyzer.completeAnalysis()
}

// MARK: - Collecting observer for one-shot classification

private final class CollectingObserver: NSObject, SNResultsObserving {
    var results: [SAClassificationResultRaw] = []
    var error: Error?

    func request(_ request: SNRequest, didProduce result: SNResult) {
        guard let classification = result as? SNClassificationResult else { return }
        appendClassificationResult(classification, to: &results)
    }

    func request(_ request: SNRequest, didFailWithError error: Error) {
        self.error = error
    }

    func requestDidComplete(_ request: SNRequest) {}
}

private func collectFileAnalysis(
    analyzer: SNAudioFileAnalyzer,
    request: SNClassifySoundRequest,
    outArray: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    outCount: UnsafeMutablePointer<Int>,
    outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let observer = CollectingObserver()
    do {
        try analyzer.add(request, withObserver: observer)
    } catch {
        outArray.pointee = nil
        outCount.pointee = 0
        return fail(outErrorMessage, defaultStatus: SA_ANALYSIS_FAILED, error: error)
    }

    analyzer.analyze()

    if let error = observer.error {
        outArray.pointee = nil
        outCount.pointee = 0
        return fail(outErrorMessage, status: SA_ANALYSIS_FAILED, message: "analysis failed: \(error.localizedDescription)")
    }

    let count = observer.results.count
    guard count > 0 else {
        outArray.pointee = nil
        outCount.pointee = 0
        return SA_OK
    }

    let buffer = UnsafeMutablePointer<SAClassificationResultRaw>.allocate(capacity: count)
    for (index, row) in observer.results.enumerated() {
        buffer.advanced(by: index).initialize(to: row)
    }
    outArray.pointee = UnsafeMutableRawPointer(buffer)
    outCount.pointee = count
    return SA_OK
}

// MARK: - Convenience file classification

@_cdecl("sa_classify_file")
public func sa_classify_file(
    _ audioPath: UnsafePointer<CChar>,
    _ outArray: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ outCount: UnsafeMutablePointer<Int>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let url = URL(fileURLWithPath: String(cString: audioPath))
    do {
        let analyzer = try SNAudioFileAnalyzer(url: url)
        let request = try builtInRequest(classifier: SA_CLASSIFIER_IDENTIFIER_VERSION1)
        return collectFileAnalysis(
            analyzer: analyzer,
            request: request,
            outArray: outArray,
            outCount: outCount,
            outErrorMessage: outErrorMessage
        )
    } catch {
        outArray.pointee = nil
        outCount.pointee = 0
        let status: Int32 = (error as NSError).domain == NSCocoaErrorDomain ? SA_AUDIO_LOAD_FAILED : SA_REQUEST_CREATE_FAILED
        return fail(outErrorMessage, defaultStatus: status, error: error)
    }
}

@_cdecl("sa_classification_results_free")
public func sa_classification_results_free(_ array: UnsafeMutableRawPointer?, _ count: Int) {
    guard let array else { return }
    let typed = array.assumingMemoryBound(to: SAClassificationResultRaw.self)
    for index in 0..<count {
        let row = typed.advanced(by: index).pointee
        freeClassificationsRaw(row.classifications, count: row.classification_count)
    }
    typed.deallocate()
}

@_cdecl("sa_classify_file_with_model")
public func sa_classify_file_with_model(
    _ audioPath: UnsafePointer<CChar>,
    _ modelPath: UnsafePointer<CChar>,
    _ outArray: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ outCount: UnsafeMutablePointer<Int>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let audioURL = URL(fileURLWithPath: String(cString: audioPath))
    let modelURL = URL(fileURLWithPath: String(cString: modelPath))
    do {
        let analyzer = try SNAudioFileAnalyzer(url: audioURL)
        let model = try MLModel(contentsOf: modelURL)
        let request = try SNClassifySoundRequest(mlModel: model)
        return collectFileAnalysis(
            analyzer: analyzer,
            request: request,
            outArray: outArray,
            outCount: outCount,
            outErrorMessage: outErrorMessage
        )
    } catch {
        outArray.pointee = nil
        outCount.pointee = 0
        let nsError = error as NSError
        let status: Int32 = if nsError.domain == NSCocoaErrorDomain {
            SA_AUDIO_LOAD_FAILED
        } else {
            SA_REQUEST_CREATE_FAILED
        }
        return fail(outErrorMessage, defaultStatus: status, error: error)
    }
}

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
