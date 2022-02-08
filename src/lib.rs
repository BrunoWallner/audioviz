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
//!```
//! use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
//! use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig, Interpolation}, stream::Stream};
//! use audioviz::distributor::{Distributor, Elapsed};
//! 
//! use std::time::Instant;
//!
//! fn main() {
//!     // captures audio from system using cpal
//!     let audio_capture = Capture::init(CaptureConfig::default()).unwrap();
//!     let audio_receiver = audio_capture.get_receiver().unwrap();
//!
//!     // smooths choppy audio data received from audio_receiver
//!     let mut distributor: Distributor<f32> = Distributor::new(44_100.0);
//! 
//!     // neccessary for distributor
//!     let mut delta_push: Instant = Instant::now();
//!     let mut delta_pop: Instant = Instant::now();
//!
//!     // spectrum visualizer stream
//!     let mut stream: Stream = Stream::new(StreamConfig::default()); 
//!     loop {
//!         if let Some(data) = audio_receiver.receive_data() {
//!             let elapsed = delta_push.elapsed().as_micros();
//!             distributor.push(&data, Elapsed::Micros(elapsed));
//!             delta_push = Instant::now();
//!         }
//!         let elapsed = delta_pop.elapsed().as_micros();
//!         let data = distributor.pop(Elapsed::Micros(elapsed));
//!         delta_pop = Instant::now();
//!         stream.push_data(data);
//! 
//!         stream.update();
//!  
//!         let frequencies = stream.get_frequencies();
//! 
//!         break; // otherwise unittest wont return
//!     }
//! }
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