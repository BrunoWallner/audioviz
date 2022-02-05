# audioviz
 Audioviz is a simple and easy to use library that helps you visualise raw audio-data

 This is done with the help of the Fast Fourier Transform algorithm,
 some frequency-space and volume normalisation and optional effects like gravity.

 There are currently only high-level abstractions for live visualisation, where
 it is consistently fed with data,
 
 but mp3 or wav file abstractions might be added in the future.
 
## Demo
![demo](./media/demo.gif)

## implementations
* [crav](https://github.com/BrunoWallner/crav)
* [audiovis](https://github.com/BrunoWallner/audiovis)
* [audiolizer](https://github.com/BrunoWallner/audiolizer)

## Features
* Fast Fourier transform via [RustFFT](https://github.com/ejmahler/RustFFT) with space and volume normalisation
* configuration that can be modified at runtime
* high-level abstraction but still possible to do everything manually
* multiple interpolation modes like cubic and linear
* system audio capturing using [cpal](https://github.com/RustAudio/cpal)
* multithreaded event based approach
* should be possible to implement in any project

## Rust features
| feature | description |
|---------|-------------|
| `cpal`  | capturing of systemd audio |
| `serde` | implementation of Serialize and Deserialize traits |
| `distributor` | helper for choppy audio-data stream smoothing |
| `spectrum` | spectrum visualisation module |

# Code Example with spectrum
```rs
use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig}, stream::Stream};
use audioviz::distributor::Distributor;
 
fn main() {
    // captures audio from system using cpal
    let audio_capture = Capture::init(CaptureConfig::default()).unwrap();
    let audio_receiver = audio_capture.get_receiver().unwrap();

    // smooths choppy audio data received from audio_receiver
    let mut distributor: Distributor<f32> = Distributor::new();

    // continuous processing of data received from capture
    let audio = Stream::init_with_capture(&capture, StreamConfig::default());
    let audio_controller: StreamController = audio.get_controller();

    // spectrum visualizer stream
    let mut stream: Stream = Stream::new(StreamConfig::default()); 

    loop {
        // stored as Vec<`spectrum::Frequency`>
        let data = stream.get_frequencies();
        /*
        do something with data ...
        */
    }
}
```

## design goals
* highly and easily configurable
* high level abstraction but preserving the possibility to do everything manually
* pretty output

### non design goals
* lightweight
* blazingly fast
* scientific accurate output
