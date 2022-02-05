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
//! # Code Example with spectrum
//! ```
//! // make sure to enable the `cpal` feature for audio capturing from system
//! use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
//! 
//! use audioviz::spectrum::stream::{Stream, StreamController};
//! use audioviz::spectrum::config::StreamConfig;
//!
//!
//! // captures audio from system using cpal
//! let capture = Capture::init(CaptureConfig::default())
//!     .unwrap();
//!
//! // continuous processing of data received from capture
//! let audio = Stream::init_with_capture(&capture, StreamConfig::default());
//! let audio_controller: StreamController = audio.get_controller();
//!
//! loop {
//!     // stored as Vec<`spectrum::Frequency`>
//!     let data = audio_controller.get_frequencies();
//!     /*
//!     do something with data ...
//!     */
//! }
//!
//! ```

/// seperates continuous audio-data to vector of single frequencies
pub mod spectrum;

/// captures audio from system using cpal
#[cfg(feature = "cpal")]
pub mod audio_capture;

#[cfg(feature = "distributor")]
pub mod distributor;