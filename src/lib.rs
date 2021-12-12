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
//! # Code Example with spectralizer
//! ```
//! use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
//! use audioviz::spectralizer::stream::{Stream, StreamController};
//! use audioviz::spectralizer::config::StreamConfig;
//!
//!
//! // captures audio from system using cpal
//! let capture = Capture::init(CaptureConfig::default());
//!
//! // continuous processing of data received from capture
//! let audio = Stream::init_with_capture(capture, StreamConfig::default());
//! let audio_controller: StreamController = audio.get_controller();
//!
//! loop {
//!     // stored as Vec<`spectralizer::Frequency`>
//!     let data = audio_controller.get_frequencies();
//!     /*
//!     do something with data ...
//!     */
//! }
//!
//! ```

/// seperates continuous audio-data to vector of single frequencies
pub mod spectralizer;

/// captures audio from system using cpal
#[cfg(feature = "cpal")]
pub mod audio_capture;
