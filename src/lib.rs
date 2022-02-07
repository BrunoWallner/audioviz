//! Audioviz is a simple and easy to use library that helps you visualise raw audio-data
//!
//! This is done with the help of the Fast Fourier Transform algorithm,
//! some frequency-space and volume normalisation and optional effects like gravity.
//!
//! There are currently only high-level abstractions for live visualisation, where
//! it is consistently fed with data,
//!
//! but mp3 or wav file abstractions might be added in the future.
//!
//!# Code Example with spectrum
//!```rs
//!use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
//!use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig}, stream::Stream};
//!use audioviz::distributor::Distributor;
//!
//!fn main() {
//!    // captures audio from system using cpal
//!    let audio_capture = Capture::init(CaptureConfig::default()).unwrap();
//!    let audio_receiver = audio_capture.get_receiver().unwrap();
//!
//!    // smooths choppy audio data received from audio_receiver
//!    let mut distributor: Distributor<f32> = Distributor::new();
//!
//!    // continuous processing of data received from capture
//!    let audio = Stream::init_with_capture(&capture, StreamConfig::default());
//!    let audio_controller: StreamController = audio.get_controller();
//!
//!    // spectrum visualizer stream
//!    let mut stream: Stream = Stream::new(StreamConfig::default()); 
//!
//!    loop {
//!        // stored as Vec<`spectrum::Frequency`>
//!        let data = stream.get_frequencies();
//!        /*
//!        do something with data ...
//!        */
//!    }
//!}
//!```

/// seperates continuous audio-data to vector of single frequencies
#[cfg(feature = "spectrum")]
pub mod spectrum;

/// captures audio from system using cpal
#[cfg(feature = "cpal")]
pub mod audio_capture;

#[cfg(feature = "distributor")]
pub mod distributor;

#[cfg(feature = "fft")]
pub mod fft;

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::path::Path;
    #[test]

    // will run cargo check for every example
    fn check_examples() {        
        let examples: &[&str] = &[
            "audio_scope",
            "audio_spectrum",
            "device_selector",
            "distributor"
        ];

        for example in examples {
            let path = Path::new("examples").join(example);
            let command = Command::new("cargo")
                .current_dir(path)
                .arg("check")
                .status()
                .unwrap();
            assert!(command.success());    
        }
    }
}