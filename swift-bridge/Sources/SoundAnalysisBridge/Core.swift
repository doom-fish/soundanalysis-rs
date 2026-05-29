import AVFoundation
import CoreML
import CoreMedia
import Foundation
import SoundAnalysis

// MARK: - Status codes

let SA_OK: Int32 = 0
let SA_INVALID_ARGUMENT: Int32 = -1
let SA_AUDIO_LOAD_FAILED: Int32 = -2
let SA_REQUEST_CREATE_FAILED: Int32 = -3
let SA_ANALYSIS_FAILED: Int32 = -4
let SA_UNKNOWN: Int32 = -99

let SA_CLASSIFIER_IDENTIFIER_VERSION1: Int32 = 1

let SA_CONSTRAINT_ENUMERATED: Int32 = 1
let SA_CONSTRAINT_RANGE: Int32 = 2

let SA_SAMPLE_FORMAT_FLOAT32: Int32 = 1
let SA_SAMPLE_FORMAT_FLOAT64: Int32 = 2
let SA_SAMPLE_FORMAT_INT16: Int32 = 3
let SA_SAMPLE_FORMAT_INT32: Int32 = 4

// MARK: - CString helpers

@inline(__always)
func ffiString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    strdup(string)
}

@_cdecl("sa_string_free")
public func sa_string_free(_ string: UnsafeMutablePointer<CChar>?) {
    guard let string else { return }
    free(string)
}

@_cdecl("sa_copy_sn_error_domain")
public func sa_copy_sn_error_domain() -> UnsafeMutablePointer<CChar>? {
    ffiString(SNErrorDomain)
}

@_cdecl("sa_double_array_free")
public func sa_double_array_free(_ array: UnsafeMutablePointer<Double>?, _: Int) {
    guard let array else { return }
    array.deallocate()
}

@inline(__always)
func bridgeError(_ message: String, status: Int32 = SA_INVALID_ARGUMENT) -> NSError {
    NSError(
        domain: "soundanalysis.bridge",
        code: Int(status),
        userInfo: [NSLocalizedDescriptionKey: message]
    )
}

@inline(__always)
func fail(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    status: Int32,
    message: String
) -> Int32 {
    outErrorMessage?.pointee = ffiString(message)
    return status
}

@inline(__always)
func fail(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    defaultStatus: Int32,
    error: Error
) -> Int32 {
    let nsError = error as NSError
    let status = nsError.code == Int(SA_INVALID_ARGUMENT) ? SA_INVALID_ARGUMENT : defaultStatus
    return fail(outErrorMessage, status: status, message: error.localizedDescription)
}

// MARK: - Layout-compatible FFI structs

public struct SAClassificationRaw {
    public var identifier: UnsafeMutablePointer<CChar>?
    public var confidence: Double
}

public struct SAClassificationResultRaw {
    public var time_start: Double
    public var time_duration: Double
    public var classifications: UnsafeMutablePointer<SAClassificationRaw>?
    public var classification_count: Int
}

public struct SATimeDurationConstraintRaw {
    public var kind: Int32
    public var rangeStartSeconds: Double
    public var rangeDurationSeconds: Double
    public var values: UnsafeMutablePointer<Double>?
    public var valueCount: Int
}

public struct SAStreamFormatRaw {
    public var sampleRate: Double
    public var channelCount: UInt32
    public var sampleFormat: Int32
    public var interleaved: Bool
}

public struct SAStreamBufferRaw {
    public var sampleFormat: Int32
    public var channelCount: UInt32
    public var frameLength: Int
    public var interleaved: Bool
    public var interleavedData: UnsafeRawPointer?
    public var planarData: UnsafePointer<UnsafeRawPointer?>?
}

// MARK: - FFI Layout Verification

/// Cross-language ABI check called from Rust's `tests/ffi_layout_tests.rs`.
///
/// Returns `true` only if the Swift `MemoryLayout` of every FFI struct matches
/// the values pinned on the Rust side via the `const _: () = assert!(...)`
/// checks in `src/ffi/mod.rs`. Rust's `size_of` includes trailing padding, so
/// it is compared against Swift's `.stride` (not `.size`). If the layouts ever
/// drift apart this returns `false` and the Rust test fails, flagging a real
/// ABI mismatch.
@_cdecl("sa_verify_ffi_layout")
public func verifyFFILayout() -> Bool {
    return MemoryLayout<SAClassificationRaw>.stride == 16
        && MemoryLayout<SAClassificationRaw>.alignment == 8
        && MemoryLayout<SAClassificationResultRaw>.stride == 32
        && MemoryLayout<SAClassificationResultRaw>.alignment == 8
        && MemoryLayout<SATimeDurationConstraintRaw>.stride == 40
        && MemoryLayout<SATimeDurationConstraintRaw>.alignment == 8
        && MemoryLayout<SAStreamFormatRaw>.stride == 24
        && MemoryLayout<SAStreamFormatRaw>.alignment == 8
        && MemoryLayout<SAStreamBufferRaw>.stride == 40
        && MemoryLayout<SAStreamBufferRaw>.alignment == 8
}

// MARK: - Object retain/release helpers

@inline(__always)
func saRetain(_ object: some AnyObject) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

@inline(__always)
func saBorrow<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as _: T.Type = T.self) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

@inline(__always)
func saRelease<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as _: T.Type) {
    Unmanaged<T>.fromOpaque(ptr).release()
}

// MARK: - Common conversions

@inline(__always)
func secondsToCMTime(_ seconds: Double) -> CMTime {
    CMTime(seconds: seconds, preferredTimescale: 1_000_000_000)
}

@inline(__always)
func copyStrings(
    _ strings: [String],
    outArray: UnsafeMutablePointer<UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?>,
    outCount: UnsafeMutablePointer<Int>
) {
    guard !strings.isEmpty else {
        outArray.pointee = nil
        outCount.pointee = 0
        return
    }

    let buffer = UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>.allocate(capacity: strings.count)
    for (index, string) in strings.enumerated() {
        buffer.advanced(by: index).initialize(to: ffiString(string))
    }
    outArray.pointee = buffer
    outCount.pointee = strings.count
}

func classificationsRaw(_ classifications: [SNClassification]) -> UnsafeMutablePointer<SAClassificationRaw>? {
    guard !classifications.isEmpty else { return nil }
    let buffer = UnsafeMutablePointer<SAClassificationRaw>.allocate(capacity: classifications.count)
    for (index, classification) in classifications.enumerated() {
        buffer.advanced(by: index).initialize(
            to: SAClassificationRaw(
                identifier: ffiString(classification.identifier),
                confidence: classification.confidence
            )
        )
    }
    return buffer
}

func freeClassificationsRaw(_ buffer: UnsafeMutablePointer<SAClassificationRaw>?, count: Int) {
    guard let buffer else { return }
    for index in 0..<count {
        if let identifier = buffer.advanced(by: index).pointee.identifier {
            free(identifier)
        }
    }
    buffer.deallocate()
}

func appendClassificationResult(
    _ result: SNClassificationResult,
    to rows: inout [SAClassificationResultRaw]
) {
    let buffer = classificationsRaw(result.classifications)
    rows.append(
        SAClassificationResultRaw(
            time_start: CMTimeGetSeconds(result.timeRange.start),
            time_duration: CMTimeGetSeconds(result.timeRange.duration),
            classifications: buffer,
            classification_count: result.classifications.count
        )
    )
}

func fillConstraintRaw(_ constraint: SNTimeDurationConstraint, outRaw: UnsafeMutableRawPointer?) -> Int32 {
    guard let outRaw else { return SA_INVALID_ARGUMENT }
    let raw = outRaw.assumingMemoryBound(to: SATimeDurationConstraintRaw.self)
    raw.pointee = SATimeDurationConstraintRaw(
        kind: 0,
        rangeStartSeconds: 0,
        rangeDurationSeconds: 0,
        values: nil,
        valueCount: 0
    )

    switch constraint {
    case let .enumeratedDurations(durations):
        raw.pointee.kind = SA_CONSTRAINT_ENUMERATED
        guard !durations.isEmpty else { return SA_OK }
        let buffer = UnsafeMutablePointer<Double>.allocate(capacity: durations.count)
        for (index, duration) in durations.enumerated() {
            buffer.advanced(by: index).initialize(to: CMTimeGetSeconds(duration))
        }
        raw.pointee.values = buffer
        raw.pointee.valueCount = durations.count
        return SA_OK
    case let .durationRange(range):
        raw.pointee.kind = SA_CONSTRAINT_RANGE
        raw.pointee.rangeStartSeconds = CMTimeGetSeconds(range.start)
        raw.pointee.rangeDurationSeconds = CMTimeGetSeconds(range.duration)
        return SA_OK
    @unknown default:
        return SA_UNKNOWN
    }
}

func builtInRequest(classifier: Int32) throws -> SNClassifySoundRequest {
    switch classifier {
    case SA_CLASSIFIER_IDENTIFIER_VERSION1:
        return try SNClassifySoundRequest(classifierIdentifier: .version1)
    default:
        throw bridgeError("unknown classifier identifier \(classifier)")
    }
}

func avAudioCommonFormat(for sampleFormat: Int32) -> AVAudioCommonFormat? {
    switch sampleFormat {
    case SA_SAMPLE_FORMAT_FLOAT32:
        return .pcmFormatFloat32
    case SA_SAMPLE_FORMAT_FLOAT64:
        return .pcmFormatFloat64
    case SA_SAMPLE_FORMAT_INT16:
        return .pcmFormatInt16
    case SA_SAMPLE_FORMAT_INT32:
        return .pcmFormatInt32
    default:
        return nil
    }
}

func bytesPerSample(for sampleFormat: Int32) -> Int? {
    switch sampleFormat {
    case SA_SAMPLE_FORMAT_FLOAT32:
        return MemoryLayout<Float>.size
    case SA_SAMPLE_FORMAT_FLOAT64:
        return MemoryLayout<Double>.size
    case SA_SAMPLE_FORMAT_INT16:
        return MemoryLayout<Int16>.size
    case SA_SAMPLE_FORMAT_INT32:
        return MemoryLayout<Int32>.size
    default:
        return nil
    }
}

func makeAVAudioFormat(from raw: SAStreamFormatRaw) -> AVAudioFormat? {
    guard raw.sampleRate > 0, raw.channelCount > 0,
          let commonFormat = avAudioCommonFormat(for: raw.sampleFormat)
    else {
        return nil
    }
    return AVAudioFormat(
        commonFormat: commonFormat,
        sampleRate: raw.sampleRate,
        channels: AVAudioChannelCount(raw.channelCount),
        interleaved: raw.interleaved
    )
}

func makePCMBuffer(from raw: SAStreamBufferRaw, format: AVAudioFormat) -> AVAudioPCMBuffer? {
    guard raw.frameLength >= 0,
          format.channelCount == AVAudioChannelCount(raw.channelCount),
          format.isInterleaved == raw.interleaved,
          format.commonFormat == avAudioCommonFormat(for: raw.sampleFormat),
          let bytesPerSample = bytesPerSample(for: raw.sampleFormat),
          let pcmBuffer = AVAudioPCMBuffer(
              pcmFormat: format,
              frameCapacity: AVAudioFrameCount(raw.frameLength)
          )
    else {
        return nil
    }

    pcmBuffer.frameLength = AVAudioFrameCount(raw.frameLength)
    let buffers = UnsafeMutableAudioBufferListPointer(pcmBuffer.mutableAudioBufferList)

    if raw.interleaved {
        guard buffers.count == 1,
              let source = raw.interleavedData,
              let destination = buffers[0].mData
        else {
            return nil
        }
        let byteCount = raw.frameLength * Int(raw.channelCount) * bytesPerSample
        memcpy(destination, source, byteCount)
        buffers[0].mDataByteSize = UInt32(byteCount)
        return pcmBuffer
    }

    guard let planarData = raw.planarData, buffers.count == Int(raw.channelCount) else {
        return nil
    }
    let byteCount = raw.frameLength * bytesPerSample
    for channel in 0..<Int(raw.channelCount) {
        guard let source = planarData[channel], let destination = buffers[channel].mData else {
            return nil
        }
        memcpy(destination, source, byteCount)
        buffers[channel].mDataByteSize = UInt32(byteCount)
    }
    return pcmBuffer
}

// MARK: - Observer boxes

public typealias SAObserverResultCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Double,
    Double,
    UnsafeMutableRawPointer?,
    Int
) -> Void

public typealias SAObserverErrorCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    UnsafeMutablePointer<CChar>?
) -> Void

public typealias SAObserverCompleteCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

final class SAObserverBox: NSObject, SNResultsObserving {
    let userInfo: UnsafeMutableRawPointer?
    let resultCallback: SAObserverResultCallback
    let errorCallback: SAObserverErrorCallback
    let completeCallback: SAObserverCompleteCallback
    let recordError: (Error) -> Void

    init(
        userInfo: UnsafeMutableRawPointer?,
        resultCallback: @escaping SAObserverResultCallback,
        errorCallback: @escaping SAObserverErrorCallback,
        completeCallback: @escaping SAObserverCompleteCallback,
        recordError: @escaping (Error) -> Void
    ) {
        self.userInfo = userInfo
        self.resultCallback = resultCallback
        self.errorCallback = errorCallback
        self.completeCallback = completeCallback
        self.recordError = recordError
    }

    func request(_ request: SNRequest, didProduce result: SNResult) {
        guard let classification = result as? SNClassificationResult else { return }
        let buffer = classificationsRaw(classification.classifications)
        resultCallback(
            userInfo,
            CMTimeGetSeconds(classification.timeRange.start),
            CMTimeGetSeconds(classification.timeRange.duration),
            UnsafeMutableRawPointer(buffer),
            classification.classifications.count
        )
        freeClassificationsRaw(buffer, count: classification.classifications.count)
    }

    func request(_ request: SNRequest, didFailWithError error: Error) {
        recordError(error)
        errorCallback(userInfo, SA_ANALYSIS_FAILED, ffiString("analysis failed: \(error.localizedDescription)"))
    }

    func requestDidComplete(_ request: SNRequest) {
        completeCallback(userInfo)
    }
}

final class SAFileAnalyzerBox: NSObject {
    let analyzer: SNAudioFileAnalyzer
    var observers: [UInt: SAObserverBox] = [:]
    var latestAnalysisError: Error?

    init(url: URL) throws {
        analyzer = try SNAudioFileAnalyzer(url: url)
    }
}

final class SAStreamAnalyzerBox: NSObject {
    let analyzer: SNAudioStreamAnalyzer
    let format: AVAudioFormat
    var observers: [UInt: SAObserverBox] = [:]
    var latestAnalysisError: Error?

    init(format: AVAudioFormat) {
        self.format = format
        analyzer = SNAudioStreamAnalyzer(format: format)
    }
}
