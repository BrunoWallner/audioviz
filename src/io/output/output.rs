//! On linux it can happen, that alsa prints to stderr
//! for this I recommend to use `https://github.com/Stebalien/gag-rs`

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::warn;
// use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc;

pub trait Anyway {
    fn anyway(&self) -> Result<(), ()>;
}
impl<T, E> Anyway for Result<T, E> {
    fn anyway(&self) -> Result<(), ()> {
        match self {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}

use super::super::Device;
use super::super::Error;

#[derive(Clone)]
pub struct OutputController {
    // data: Arc<Mutex<Vec<f32>>>,
    sender: mpsc::Sender<f32>,
}
impl OutputController {
    pub fn push_data(&self, data: Vec<f32>) {
        for d in data {
            let _ = self.sender.send(d);
        }
    }
}

pub struct Output {
    host: cpal::platform::Host,
    // stream must stay in scope
    stream: Option<cpal::Stream>,
}
impl Output {
    pub fn new() -> Self {
        let host = cpal::default_host();

        return Self { host, stream: None };
    }
    /// returns: `channel_count`, `sampling_rate` and `CaptureReceiver`
    pub fn init(&mut self, device: &Device) -> Result<(u16, u32, OutputController), Error> {
        let (sender, receiver) = mpsc::channel();
        let output_controller = OutputController { sender };

        let (channel_count, stream, sampling_rate) =
            match stream_audio_to_distributor(&self.host, receiver, device) {
                Ok(s) => s,
                Err(e) => return Err(e),
            };

        self.stream = Some(stream);

        Ok((channel_count, sampling_rate, output_controller))
    }

    pub fn fetch_devices(&self) -> Result<Vec<String>, Error> {
        let devices = match self.host.devices() {
            Ok(d) => d,
            Err(e) => match e {
                cpal::DevicesError::BackendSpecific { err } => {
                    let cpal::BackendSpecificError { description } = err;
                    return Err(Error::BackendSpecific(description));
                }
            },
        };
        let devices: Vec<String> = devices
            .into_iter()
            .map(|dev| dev.name().unwrap_or_else(|_| String::from("invalid")))
            .collect();

        Ok(devices)
    }
}

fn stream_audio_to_distributor(
    host: &cpal::platform::Host,
    receiver: mpsc::Receiver<f32>,
    device: &Device,
    // returns channel-count, stream and sampling-rate
) -> Result<(u16, cpal::Stream, u32), Error> {
    let device = match device {
        &Device::DefaultInput => match host.default_input_device() {
            Some(d) => d,
            None => return Err(Error::DeviceNotFound),
        },
        &Device::DefaultOutput => match host.default_output_device() {
            Some(d) => d,
            None => return Err(Error::DeviceNotFound),
        },
        &Device::Id(id) => match host.output_devices() {
            Ok(mut devices) => match devices.nth(id) {
                Some(d) => d,
                None => return Err(Error::DeviceNotFound),
            },
            Err(_) => return Err(Error::DeviceNotFound),
        },
    };

    let config: cpal::SupportedStreamConfig = match device.default_output_config() {
        Ok(c) => c,
        Err(_) => return Err(Error::DeviceNotAvailable),
    };

    let channel_count = config.channels();
    let sampling_rate = config.sample_rate();

    #[allow(unused_must_use)]
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &_| {
                for sample in data {
                    if let Ok(data) = receiver.try_recv() {
                        *sample = data;
                    } else {
                        *sample = 0.0;
                    }
                }
            },
            |e| warn!("error occurred on capture-stream: {}", e),
        ),
        _ => return Err(Error::UnsupportedConfig),
    };

    let stream = match stream {
        Ok(s) => s,
        Err(e) => match e {
            cpal::BuildStreamError::DeviceNotAvailable => return Err(Error::DeviceNotAvailable),
            cpal::BuildStreamError::StreamConfigNotSupported => {
                return Err(Error::UnsupportedConfig)
            }
            cpal::BuildStreamError::BackendSpecific { err } => {
                return Err(Error::BackendSpecific(err.to_string()))
            }
            err => return Err(Error::BackendSpecific(err.to_string())),
        },
    };

    stream.play().unwrap();

    Ok((channel_count, stream, sampling_rate.0))
}
