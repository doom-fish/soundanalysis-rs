#![cfg(feature = "stream")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use soundanalysis::{
    AudioStreamAnalyzer, AudioStreamFormat, ClassifySoundRequest, PcmBuffer, PcmSampleFormat,
    ResultsObserverFns,
};

const SAMPLE_RATE: f64 = 16_000.0;

fn tone(frames: u16) -> Vec<f32> {
    (0..frames)
        .map(|frame| {
            let phase = f64::from(frame) * 440.0 * std::f64::consts::TAU / SAMPLE_RATE;
            #[allow(clippy::cast_possible_truncation)]
            let sample = (phase.sin() * 0.5) as f32;
            sample
        })
        .collect()
}

fn wait_for(counter: &AtomicUsize, minimum: usize) -> usize {
    let deadline = Instant::now() + Duration::from_secs(10);
    while counter.load(Ordering::SeqCst) < minimum && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    counter.load(Ordering::SeqCst)
}

#[test]
fn stream_analyzer_classifies_interleaved_and_planar_pcm() -> Result<(), Box<dyn std::error::Error>>
{
    let left = tone(32_000);
    let right = left.clone();
    for interleaved in [true, false] {
        let format = AudioStreamFormat::new(SAMPLE_RATE, 2, PcmSampleFormat::Float32, interleaved)?;
        let mut analyzer = AudioStreamAnalyzer::new(format)?;
        let mut request = ClassifySoundRequest::version1()?;
        request.set_window_duration(1.0)?;
        let results = Arc::new(AtomicUsize::new(0));
        let completions = Arc::new(AtomicUsize::new(0));
        analyzer.add_request(
            &request,
            ResultsObserverFns::new({
                let results = Arc::clone(&results);
                move |_, _| {
                    results.fetch_add(1, Ordering::SeqCst);
                }
            })
            .on_complete({
                let completions = Arc::clone(&completions);
                move |_| {
                    completions.fetch_add(1, Ordering::SeqCst);
                }
            }),
        )?;

        if interleaved {
            let samples: Vec<f32> = left
                .iter()
                .zip(&right)
                .flat_map(|(left, right)| [*left, *right])
                .collect();
            analyzer.analyze_audio_buffer(
                PcmBuffer::InterleavedF32 {
                    samples: &samples,
                    channels: 2,
                },
                0,
            )?;
        } else {
            let planes = [left.as_slice(), right.as_slice()];
            analyzer.analyze_audio_buffer(PcmBuffer::PlanarF32(&planes), 0)?;
        }
        analyzer.complete_analysis();

        assert_eq!(wait_for(&completions, 1), 1, "interleaved={interleaved}");
        assert!(wait_for(&results, 1) > 0, "interleaved={interleaved}");
    }
    Ok(())
}
