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
//!     let mut distributor: Distributor<f32> = Distributor::new(44_100.0, 64);
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
    
    #[cfg(feature = "distributor")]
    use crate::distributor::Distributor;

    #[cfg(feature = "fft")]
    use crate::fft;

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

    #[cfg(feature = "distributor")]
    #[cfg(feature = "std")]
    #[test]
    fn distributor() {
        use std::{time::Duration, thread::sleep};
        use Distributor;

        let estimated_data_rate: f64 = 8.0 * 1000.0 / 5.0;
        let mut distributor: Distributor<u128> = Distributor::new(estimated_data_rate, 32);

        let mut counter: u128 = 0;
        'distribution: loop {
            if counter % 5 == 0 {
                let mut buffer: Vec<u128> = Vec::new();
                for _ in 0..=8 {
                    buffer.push(0);
                }

                distributor.push_auto(&buffer);
            }

            let data = distributor.pop_auto();
            let buf_len = distributor.clone_buffer().len();

            // if sample rate is fully known with 2 pushes
            if counter >= 10 {
                assert!(data.len() > 0);
                assert!(buf_len <= 16);
            }

            counter += 1;
            sleep(Duration::from_millis(1));

            if counter > 100 {
                break 'distribution;
            }
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
    
        let fft = fft::process(&buffer);
    
        assert_eq!(
            fft,
            vec![
                9.78363,
                2.9537609,
                1.4024371,
                0.95359206,
                0.74589825,
                0.63307375,
                0.569189,
                0.5359103,         
            ]
        )
    }


}