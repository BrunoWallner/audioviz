//! Captures audio from system
//!
//! then sends the data to the distributor which distributes one big buffer into multiple smaller ones
//!
//! this increases overall smoothness at the cost of increased latency
//!
//! On linux it can happen, that alsa prints to stderr
//! for this I recommend to use `https://github.com/Stebalien/gag-rs`

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::warn;
use std::sync::mpsc;

use super::super::Device;
use super::super::Error;
use super::converter;

pub struct InputController {
    receiver: mpsc::Receiver<Vec<f32>>,
}
impl InputController {
    pub fn pull_data(&self) -> Vec<f32> {
        let mut data = Vec::new();
        while let Ok(mut d) = self.receiver.try_recv() {
            data.append(&mut d);
        }
        data
    }
}

pub struct Input {
    host: cpal::platform::Host,
    // stream must stay in scope
    stream: Option<cpal::Stream>,
}
impl Input {
    pub fn new() -> Self {
        let host = cpal::default_host();

        return Self { host, stream: None };
    }
    /// returns: `channel_count`, `sampling_rate` and `CaptureReceiver`
    pub fn init(&mut self, device: &Device) -> Result<(u16, u32, InputController), Error> {
        let (sender, receiver) = mpsc::channel();
        let input_controller = InputController { receiver };

        let (channel_count, stream, sampling_rate) =
            match stream_audio_to_distributor(&self.host, sender, device) {
                Ok(s) => s,
                Err(e) => return Err(e),
            };

        self.stream = Some(stream);

        Ok((channel_count, sampling_rate, input_controller))
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
    sender: mpsc::Sender<Vec<f32>>,
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
        &Device::Id(id) => match host.input_devices() {
            Ok(mut devices) => match devices.nth(id) {
                Some(d) => d,
                None => return Err(Error::DeviceNotFound),
            },
            Err(_) => return Err(Error::DeviceNotFound),
        },
    };

    let config: cpal::SupportedStreamConfig = match device.default_input_config() {
        Ok(c) => c,
        Err(_) => return Err(Error::DeviceNotAvailable),
    };

    let channel_count = config.channels();
    let sampling_rate = config.sample_rate();

    #[allow(unused_must_use)]
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                let _ = sender.send(data.to_vec());
            },
            |e| warn!("error occurred on capture-stream: {}", e),
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| {
                let data = converter::i16_to_f32(data);
                let _ = sender.send(data);
            },
            |e| warn!("error occurred on capture-stream: {}", e),
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _: &_| {
                let data = converter::u16_to_f32(data);
                let _ = sender.send(data);
            },
            |e| warn!("error occurred on capture-stream: {}", e),
        ),
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
