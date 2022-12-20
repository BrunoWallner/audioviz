//! Audioviz is a simple and easy to use library that helps you visualise raw audio-data
//!
//! ### It consists of multiple modules:
//! - [**fft**](./fft/index.html) Fast Fourier Transform algorithm, which transforms audio-data to a representation in the frequency domain.
//! - [**spectrum**](./spectrum/index.html) high level and easier to use abstraction over [**fft**](./fft/index.html)
//! - [**distributor**](./distributor/index.html) distributes big buffers into smaller ones.
//!   Results in much smoother output of `distributor` when applied.
//! - [**audio_capture**](./audio_capture/index.html) captures system audio using [CPAL](https://github.com/RustAudio/cpal).
//!
//!# Code Example with spectrum
//!```
//! use audioviz::io::{Input, Device};
//! use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig, Interpolation}, stream::Stream};
//!
//! // captures audio from system using cpal
//! let mut audio_input = Input::new();
//! let (channel_count, sampling_rate, input_controller) = audio_input.init(&Device::DefaultInput).unwrap();
//!
//! // spectrum visualizer stream
//! let mut stream: Stream = Stream::new(StreamConfig::default());
//! loop {
//!     if let Some(data) = input_controller.pull_data() {
//!         stream.push_data(data);
//!         stream.update();
//!     }
//!
//!     let frequencies = stream.get_frequencies();
//!
//!     break; // otherwise unittest wont return
//! }
//!```

/// high level and easier to use abstraction over [**fft**](./fft/index.html)
#[cfg(feature = "spectrum")]
pub mod spectrum;

#[cfg(feature = "lissajous")]
pub mod lissajous;

#[cfg(feature = "cpal")]
pub mod io;

#[cfg(feature = "processor")]
pub mod processor;

#[cfg(feature = "fft")]
pub mod fft;

pub mod utils;

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    #[cfg(feature = "fft")]
    use crate::fft;

    #[test]
    // will run cargo check for every example
    fn check_examples() {
        let examples: &[&str] = &["audio_scope", "audio_spectrum", "device_selector", "osc"];

        for example in examples {
            let path = Path::new("examples").join(example);
            let command = Command::new("cargo")
                .current_dir(path)
                .arg("check")
                .arg("--quiet")
                .status()
                .unwrap();
            assert!(command.success(), "failed at {} example", example);
        }
    }

    #[cfg(feature = "fft")]
    #[test]
    fn fft() {
        let mut buffer: Vec<f32> = Vec::new();

        let mut x: f32 = 0.0;
        for _ in 0..16 {
            buffer.push(x.sin());
            x += 0.1;
        }

        let fft = fft::forward(&buffer);
        let fft = fft::normalize(&fft);
        let fft = fft::remove_mirroring(&fft);

        assert_eq!(
            fft,
            vec![
                9.78363, 2.9537609, 1.4024371, 0.95359206, 0.74589825, 0.63307375, 0.569189,
                0.5359103, 0.52553797
            ]
        )
    }
}
