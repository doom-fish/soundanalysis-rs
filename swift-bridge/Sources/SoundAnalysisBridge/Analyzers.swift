import AVFoundation
import CoreML
import CoreMedia
import Foundation
import SoundAnalysis

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

