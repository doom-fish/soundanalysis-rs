import AVFoundation
import CoreML
import CoreMedia
import Foundation
import SoundAnalysis

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

