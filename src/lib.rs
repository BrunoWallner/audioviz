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
//! use audioviz::audio_capture::capture::Capture;
//! use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig, Interpolation}, stream::Stream};
//! use audioviz::distributor::Distributor;
//!
//! fn main() {
//!     // captures audio from system using cpal
//!     let audio_capture = Capture::init("default").unwrap();
//!     let audio_receiver = audio_capture.get_receiver().unwrap();
//!
//!     // smooths choppy audio data received from audio_receiver
//!     let mut distributor: Distributor<f32> = Distributor::new(44_100.0, Some(8128));
//!
//!     // spectrum visualizer stream
//!     let mut stream: Stream = Stream::new(StreamConfig::default()); 
//!     loop {
//!         if let Some(data) = audio_receiver.receive_data() {
//!             distributor.push_auto(&data);
//!         }
//!         let data = distributor.pop_auto(None);
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

/// high level and easier to use abstraction over [**fft**](./fft/index.html)
#[cfg(feature = "spectrum")]
pub mod spectrum;

#[cfg(feature = "cpal")]
pub mod audio_capture;

#[cfg(feature = "distributor")]
pub mod distributor;

#[cfg(feature = "fft")]
pub mod fft;

pub mod utils;

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
                .arg("--quiet")
                .status()
                .unwrap();
            assert!(command.success(), "failed at {} example", example);    
        }
    }

    #[cfg(feature = "distributor")]
    #[cfg(feature = "std")]
    #[test]
    fn distributor() {
        use std::{time::Duration, thread::sleep};
        use Distributor;

        let estimated_data_rate: f64 = 8.0 * 1000.0 / 5.0;
        let mut distributor: Distributor<u128> = Distributor::new(estimated_data_rate, Some(16));

        let mut counter: u128 = 0;
        'distribution: loop {
            if counter % 5 == 0 {
                let mut buffer: Vec<u128> = Vec::new();
                for _ in 0..=8 {
                    buffer.push(0);
                }

                distributor.push_auto(&buffer);
            }

            let data = distributor.pop_auto(None);
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