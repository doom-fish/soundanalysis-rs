//! `SNAudioStreamAnalyzer` wrapper.

use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;

use crate::error::{from_swift, SAError};
use crate::ffi;
use crate::observer::{
    box_observer, clear_observers, drop_observer, observer_complete_trampoline,
    observer_error_trampoline, observer_result_trampoline, request_key, ObserverMap,
    ResultsObserver,
};
use crate::request::ClassifySoundRequest;

/// Supported PCM sample formats for [`AudioStreamAnalyzer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmSampleFormat {
    Float32,
    Float64,
    Int16,
    Int32,
}

impl PcmSampleFormat {
    const fn as_ffi(self) -> i32 {
        match self {
            Self::Float32 => ffi::sample_format::FLOAT32,
            Self::Float64 => ffi::sample_format::FLOAT64,
            Self::Int16 => ffi::sample_format::INT16,
            Self::Int32 => ffi::sample_format::INT32,
        }
    }
}

/// Audio format description for `SNAudioStreamAnalyzer`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioStreamFormat {
    pub sample_rate: f64,
    pub channel_count: usize,
    pub sample_format: PcmSampleFormat,
    pub interleaved: bool,
}

impl AudioStreamFormat {
    pub fn new(
        sample_rate: f64,
        channel_count: usize,
        sample_format: PcmSampleFormat,
        interleaved: bool,
    ) -> Result<Self, SAError> {
        if !(sample_rate.is_finite() && sample_rate > 0.0) {
            return Err(SAError::InvalidArgument(
                "sample_rate must be finite and > 0".into(),
            ));
        }
        if channel_count == 0 {
            return Err(SAError::InvalidArgument(
                "channel_count must be greater than zero".into(),
            ));
        }

        Ok(Self {
            sample_rate,
            channel_count,
            sample_format,
            interleaved,
        })
    }

    fn to_raw(self) -> Result<ffi::StreamFormatRaw, SAError> {
        Ok(ffi::StreamFormatRaw {
            sample_rate: self.sample_rate,
            channel_count: u32::try_from(self.channel_count).map_err(|_| {
                SAError::InvalidArgument("channel_count does not fit in u32".into())
            })?,
            sample_format: self.sample_format.as_ffi(),
            interleaved: self.interleaved,
        })
    }
}

/// PCM buffer layouts accepted by [`AudioStreamAnalyzer::analyze_audio_buffer`].
#[derive(Debug, Clone, Copy)]
pub enum PcmBuffer<'a> {
    InterleavedF32 {
        samples: &'a [f32],
        channels: usize,
    },
    InterleavedF64 {
        samples: &'a [f64],
        channels: usize,
    },
    InterleavedI16 {
        samples: &'a [i16],
        channels: usize,
    },
    InterleavedI32 {
        samples: &'a [i32],
        channels: usize,
    },
    PlanarF32(&'a [&'a [f32]]),
    PlanarF64(&'a [&'a [f64]]),
    PlanarI16(&'a [&'a [i16]]),
    PlanarI32(&'a [&'a [i32]]),
}

impl<'a> PcmBuffer<'a> {
    fn sample_format(self) -> PcmSampleFormat {
        match self {
            Self::InterleavedF32 { .. } | Self::PlanarF32(_) => PcmSampleFormat::Float32,
            Self::InterleavedF64 { .. } | Self::PlanarF64(_) => PcmSampleFormat::Float64,
            Self::InterleavedI16 { .. } | Self::PlanarI16(_) => PcmSampleFormat::Int16,
            Self::InterleavedI32 { .. } | Self::PlanarI32(_) => PcmSampleFormat::Int32,
        }
    }

    fn interleaved(self) -> bool {
        matches!(
            self,
            Self::InterleavedF32 { .. }
                | Self::InterleavedF64 { .. }
                | Self::InterleavedI16 { .. }
                | Self::InterleavedI32 { .. }
        )
    }

    fn channel_count(self) -> usize {
        match self {
            Self::InterleavedF32 { channels, .. }
            | Self::InterleavedF64 { channels, .. }
            | Self::InterleavedI16 { channels, .. }
            | Self::InterleavedI32 { channels, .. } => channels,
            Self::PlanarF32(channels) => channels.len(),
            Self::PlanarF64(channels) => channels.len(),
            Self::PlanarI16(channels) => channels.len(),
            Self::PlanarI32(channels) => channels.len(),
        }
    }

    fn frame_length(self) -> Result<usize, SAError> {
        match self {
            Self::InterleavedF32 { samples, channels } => {
                interleaved_frame_length(samples.len(), channels)
            }
            Self::InterleavedF64 { samples, channels } => {
                interleaved_frame_length(samples.len(), channels)
            }
            Self::InterleavedI16 { samples, channels } => {
                interleaved_frame_length(samples.len(), channels)
            }
            Self::InterleavedI32 { samples, channels } => {
                interleaved_frame_length(samples.len(), channels)
            }
            Self::PlanarF32(channels) => planar_frame_length(channels),
            Self::PlanarF64(channels) => planar_frame_length(channels),
            Self::PlanarI16(channels) => planar_frame_length(channels),
            Self::PlanarI32(channels) => planar_frame_length(channels),
        }
    }

    fn to_raw(
        self,
        expected: AudioStreamFormat,
        planar_ptrs: &mut Vec<*const c_void>,
    ) -> Result<ffi::StreamBufferRaw, SAError> {
        let sample_format = self.sample_format();
        if sample_format != expected.sample_format {
            return Err(SAError::InvalidArgument(
                "PCM buffer sample format does not match stream analyzer format".into(),
            ));
        }
        if self.interleaved() != expected.interleaved {
            return Err(SAError::InvalidArgument(
                "PCM buffer interleaving does not match stream analyzer format".into(),
            ));
        }
        if self.channel_count() != expected.channel_count {
            return Err(SAError::InvalidArgument(
                "PCM buffer channel count does not match stream analyzer format".into(),
            ));
        }

        let frame_length = self.frame_length()?;
        let channel_count = u32::try_from(expected.channel_count)
            .map_err(|_| SAError::InvalidArgument("channel_count does not fit in u32".into()))?;

        let mut raw = ffi::StreamBufferRaw {
            sample_format: sample_format.as_ffi(),
            channel_count,
            frame_length,
            interleaved: expected.interleaved,
            interleaved_data: ptr::null(),
            planar_data: ptr::null(),
        };

        match self {
            Self::InterleavedF32 { samples, .. } => raw.interleaved_data = samples.as_ptr().cast(),
            Self::InterleavedF64 { samples, .. } => raw.interleaved_data = samples.as_ptr().cast(),
            Self::InterleavedI16 { samples, .. } => raw.interleaved_data = samples.as_ptr().cast(),
            Self::InterleavedI32 { samples, .. } => raw.interleaved_data = samples.as_ptr().cast(),
            Self::PlanarF32(channels) => {
                planar_ptrs.extend(channels.iter().map(|channel| channel.as_ptr().cast()));
                raw.planar_data = planar_ptrs.as_ptr();
            }
            Self::PlanarF64(channels) => {
                planar_ptrs.extend(channels.iter().map(|channel| channel.as_ptr().cast()));
                raw.planar_data = planar_ptrs.as_ptr();
            }
            Self::PlanarI16(channels) => {
                planar_ptrs.extend(channels.iter().map(|channel| channel.as_ptr().cast()));
                raw.planar_data = planar_ptrs.as_ptr();
            }
            Self::PlanarI32(channels) => {
                planar_ptrs.extend(channels.iter().map(|channel| channel.as_ptr().cast()));
                raw.planar_data = planar_ptrs.as_ptr();
            }
        }

        Ok(raw)
    }
}

fn interleaved_frame_length(sample_len: usize, channels: usize) -> Result<usize, SAError> {
    if channels == 0 {
        return Err(SAError::InvalidArgument(
            "interleaved buffers need at least one channel".into(),
        ));
    }
    if sample_len % channels != 0 {
        return Err(SAError::InvalidArgument(
            "interleaved samples length must be divisible by channel count".into(),
        ));
    }
    Ok(sample_len / channels)
}

fn planar_frame_length<T>(channels: &[&[T]]) -> Result<usize, SAError> {
    let Some((first, rest)) = channels.split_first() else {
        return Err(SAError::InvalidArgument(
            "planar buffers need at least one channel".into(),
        ));
    };
    let frame_length = first.len();
    if rest.iter().any(|channel| channel.len() != frame_length) {
        return Err(SAError::InvalidArgument(
            "all planar channels must have the same frame length".into(),
        ));
    }
    Ok(frame_length)
}

/// Safe wrapper around `SNAudioStreamAnalyzer`.
#[derive(Debug)]
pub struct AudioStreamAnalyzer {
    ptr: NonNull<c_void>,
    format: AudioStreamFormat,
    observers: ObserverMap,
}

impl AudioStreamAnalyzer {
    pub fn new(format: AudioStreamFormat) -> Result<Self, SAError> {
        let raw = format.to_raw()?;
        let mut analyzer = ptr::null_mut();
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_audio_stream_analyzer_create(
                std::ptr::from_ref(&raw).cast(),
                &mut analyzer,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }

        Ok(Self {
            ptr: NonNull::new(analyzer).expect("sa_audio_stream_analyzer_create returned null"),
            format,
            observers: ObserverMap::default(),
        })
    }

    #[must_use]
    pub fn format(&self) -> AudioStreamFormat {
        self.format
    }

    pub fn add_request<O>(
        &mut self,
        request: &ClassifySoundRequest,
        observer: O,
    ) -> Result<(), SAError>
    where
        O: ResultsObserver + 'static,
    {
        let key = request_key(request);
        if self.observers.contains_key(&key) {
            return Err(SAError::InvalidArgument(
                "request already added to analyzer".into(),
            ));
        }

        let observer_ptr = box_observer(request, observer);
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_audio_stream_analyzer_add_request(
                self.ptr.as_ptr(),
                request.as_raw(),
                observer_ptr,
                observer_result_trampoline,
                observer_error_trampoline,
                observer_complete_trampoline,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { drop_observer(observer_ptr) };
            return Err(unsafe { from_swift(status, err) });
        }

        self.observers.insert(key, observer_ptr.cast());
        Ok(())
    }

    pub fn remove_request(&mut self, request: &ClassifySoundRequest) {
        unsafe { ffi::sa_audio_stream_analyzer_remove_request(self.ptr.as_ptr(), request.as_raw()) };
        if let Some(observer) = self.observers.remove(&request_key(request)) {
            unsafe { drop_observer(observer.cast()) };
        }
    }

    pub fn remove_all_requests(&mut self) {
        unsafe { ffi::sa_audio_stream_analyzer_remove_all_requests(self.ptr.as_ptr()) };
        clear_observers(&mut self.observers);
    }

    pub fn analyze_audio_buffer(
        &mut self,
        buffer: PcmBuffer<'_>,
        audio_frame_position: i64,
    ) -> Result<(), SAError> {
        if audio_frame_position < 0 {
            return Err(SAError::InvalidArgument(
                "audio_frame_position must be non-negative".into(),
            ));
        }

        let mut planar_ptrs = Vec::new();
        let raw = buffer.to_raw(self.format, &mut planar_ptrs)?;
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_audio_stream_analyzer_analyze_audio_buffer(
                self.ptr.as_ptr(),
                std::ptr::from_ref(&raw).cast(),
                audio_frame_position,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn complete_analysis(&mut self) {
        unsafe { ffi::sa_audio_stream_analyzer_complete_analysis(self.ptr.as_ptr()) };
    }
}

impl Drop for AudioStreamAnalyzer {
    fn drop(&mut self) {
        if !self.observers.is_empty() {
            unsafe { ffi::sa_audio_stream_analyzer_remove_all_requests(self.ptr.as_ptr()) };
            clear_observers(&mut self.observers);
        }
        unsafe { ffi::sa_audio_stream_analyzer_release(self.ptr.as_ptr()) };
    }
}
