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
