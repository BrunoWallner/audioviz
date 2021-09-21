# audioviz
A modular and simple libary to make raw and realtime audiodata captured for example from [cpal](https://github.com/RustAudio/cpal) visually more appealing.

It is not intended for scientifically usecases.

## demo
A very simple demonstration implemented with [tui](https://github.com/fdehau/tui-rs) and [cpal](https://github.com/RustAudio/cpal)
![](/media/demo.gif)

## Features
* Fast Fourier transform via [RustFFT](https://github.com/awelkie/RustFFT) with space and volume normalisation
* configuration that can be live modified at runtime
* buffering for smoothing over time
* smoothing
* resolution control
* configurable refresh rate
* scalable custom frequencies
* very simple interface
* only about 350 lines of code
* multithreaded event based approach
* should be able to be implemented in any project

## Example with cpal
```rs
use audioviz::*;
use std::thread;
use std::sync::mpsc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // creates the AudioStream object, which handles the interface
    let audio_stream = AudioStream::init(
        // using the default configuration
        Config {
            ..Default::default()
        }
    );

    // getting the event_sender of audio_stream for easy implementation
    let event_sender = audio_stream.get_event_sender();

    // initiating the audio sending thread, that captures audio using cpal and then sends it to audio_stream via the event_sender
    // I actually dont fully know how to use cpal so it could be partially wrong but it works at least a bit
    let event_sender_clone = event_sender.clone();
    thread::spawn(move || {
        let host = cpal::default_host();

        let device = host.default_output_device().unwrap();

        let device_config =  device.default_output_config().unwrap();

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

The received data is stored via `Vec<f32>`