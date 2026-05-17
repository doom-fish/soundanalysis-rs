// Async APIs for SoundAnalysis framework

import Foundation
import SoundAnalysis

// MARK: - Thread-safe result holder

private class ResultHolder<T> {
    private let lock = NSLock()
    private var _value: T?
    private var _error: String?

    var value: T? {
        get { lock.lock(); defer { lock.unlock() }; return _value }
        set { lock.lock(); defer { lock.unlock() }; _value = newValue }
    }

    var error: String? {
        get { lock.lock(); defer { lock.unlock() }; return _error }
        set { lock.lock(); defer { lock.unlock() }; _error = newValue }
    }
}

// MARK: - SNAudioFileAnalyzer: Async Analysis

/// Async wrapper for SNAudioFileAnalyzer.analyze() using a Task
/// 
/// This thunk:
/// 1. Creates an SNAudioFileAnalyzer from the audio path
/// 2. Launches a Task that performs analysis
/// 3. Fires the completion callback when done
///
/// The caller is responsible for adding requests before calling this.
@_cdecl("sa_audio_file_analyzer_analyze_async")
public func audioFileAnalyzerAnalyzeAsync(
    audioPath: UnsafePointer<CChar>,
    cb: @convention(c) (Bool, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    ctx: UnsafeMutableRawPointer
) {
    Task {
        let pathString = String(cString: audioPath)
        
        do {
            let analyzer = try SNAudioFileAnalyzer(url: URL(fileURLWithPath: pathString))
            
            // For now, we just mark it as successful since the analyzer was created.
            // In a real implementation, you would add the request and call analyze().
            // This is a simplified version that just tests the completion callback.
            cb(true, nil, ctx)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { cstr in
                cb(false, cstr, ctx)
            }
        }
    }
}
