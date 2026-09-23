import Foundation
import SoundAnalysis

public typealias SAAnalyzeCompletionCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    Bool,
    UnsafeMutablePointer<CChar>?
) -> Void

@_cdecl("sa_audio_file_analyzer_analyze_async")
public func sa_audio_file_analyzer_analyze_async(
    _ analyzer: UnsafeMutableRawPointer?,
    _ completion: @escaping SAAnalyzeCompletionCallback,
    _ context: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let analyzer else {
        return fail(outErrorMessage, status: SA_INVALID_ARGUMENT, message: "null analyzer")
    }
    let box: SAFileAnalyzerBox = saBorrow(analyzer, as: SAFileAnalyzerBox.self)
    guard box.beginAnalysis() else {
        return fail(outErrorMessage, status: SA_ANALYSIS_FAILED, message: "analysis already in progress")
    }
    box.analyzer.analyze { reachedEndOfFile in
        if let error = box.finishAnalysis() {
            completion(
                context,
                SA_ANALYSIS_FAILED,
                reachedEndOfFile,
                ffiString("analysis failed: \(error.localizedDescription)")
            )
        } else {
            completion(context, SA_OK, reachedEndOfFile, nil)
        }
    }
    return SA_OK
}
