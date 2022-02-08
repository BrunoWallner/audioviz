# audioviz
 Audioviz is a simple and easy to use library that helps you visualise raw audio-data

 This is done with the help of the Fast Fourier Transform algorithm,
 some frequency-space and volume normalisation and optional effects like gravity.

## Features
* Fast Fourier transform via [RustFFT](https://github.com/ejmahler/RustFFT) with space and volume normalisation
* high-level abstraction but still possible to do everything manually
* multiple interpolation modes like cubic and linear
* system audio capturing using [cpal](https://github.com/RustAudio/cpal)
* should be possible to implement in any project
* modular design

## Rust features
| feature | description |
|---------|-------------|
| `cpal`  | capturing of systemd audio |
| `serde` | implementation of Serialize and Deserialize traits |
| `distributor` | helper for choppy audio-data stream smoothing |
| `spectrum` | spectrum visualisation module |
| `fft` | Fast Fourier Transform algorithm |

# Examples
Examples can be found [here](examples/) or in the documentation 