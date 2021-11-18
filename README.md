# audioviz
A modular and simple libary to make raw audiodata visually more appealing in realtime.

It is not intended for scientific usecases.

## implementations
* [crav](https://github.com/BrunoWallner/crav)
* [audiovis](https://github.com/BrunoWallner/audiovis)
* [audiolizer](https://github.com/BrunoWallner/audiolizer)

## Features
* Fast Fourier transform via [RustFFT](https://github.com/awelkie/RustFFT) with space and volume normalisation
* configuration that can be modified at runtime
* configurable amount of 'bars'
* configurable refresh rate
* eq that allows you to manually distribute frequency ranges
* configurable FFT resolution
* very simple "interface"
* multithreaded event based approach
* should be possible to implement in any project

## Example with cpal
```rs
use audioviz::*;
use std::thread;
use std::sync::mpsc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // creates the AudioStream object, which is the main interface
    let audio_stream = AudioStream::init(
        // using the default configuration
        Config {
            ..Default::default()
        }
    );

    // getting the event_sender of audio_stream for easy implementation
    let event_sender = audio_stream.get_event_sender();

    // initiating the audio sending thread, that captures audio using cpal and then sends it to audio_stream via the event_sender
    let event_sender_clone = event_sender.clone();
    thread::spawn(move || {
        let host = cpal::default_host();

        let device = host.default_output_device().unwrap();

        // as of version 3.1 it must be 1 channel
        let device_config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(44_100),
            buffer_size: cpal::BufferSize::Fixed(1000)
        };

        let stream = match device_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &device_config.into(),
                move |data, _: &_| handle_input_data_f32(data, event_sender_clone.clone()),
                err_fn,
            ).unwrap(),
            other => {
                panic!("Unsupported sample format {:?}", other);
            }
        };

        stream.play().unwrap();

        // parks the thread so stream.play() does not get dropped and stops
        thread::park();
    });

    // receives calculated and converted data from audio_stream
    loop {
        // Method 1:
        let data = audio_stream.get_audio_data();
        
        // Method 2:
        let (tx, rx) = mpsc::channel();
        event_sender.send(Event::RequestData(tx)).unwrap();
        let data = rx.recv().unwrap();

        // Do something with data...
    }

}

// functions for cpal
fn handle_input_data_f32(data: &[f32], sender: mpsc::Sender<audioviz::Event>) {
    // sends the raw data to audio_stream via the event_sender
    sender.send(audioviz::Event::SendData(data.to_vec())).unwrap();
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
```

The received data is stored in a vector of 32 bit floating points, arranged in ascending frequencies.