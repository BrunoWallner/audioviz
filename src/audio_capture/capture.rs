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
use std::thread;

use super::converter;

#[derive(Clone, Debug)]
pub enum Error {
    DeviceNotFound,
    DeviceNotAvailable,
    UnsupportedConfig,
    BackendSpecific(String),
}

#[derive(Clone, Debug)]
enum CaptureEvent {
    SendData(Vec<f32>),
    ReceiveData(mpsc::Sender<Option<Vec<f32>>>),
}

#[derive(Clone, Debug)]
pub enum Device {
    DefaultInput,
    DefaultOutput,
    Id(usize),
}

pub struct CaptureReceiver {
    sender: mpsc::Sender<CaptureEvent>,
}
impl CaptureReceiver {
    #[allow(unused_must_use)]
    pub fn receive_data(&self) -> Option<Vec<f32>> {
        let (sender, receiver) = mpsc::channel();
        self.sender.send(CaptureEvent::ReceiveData(sender));
        match receiver.recv() {
            Ok(val) => val,
            Err(_) => None,
        }
    }
}

pub struct Capture {
    pub channel_count: Option<u16>,
    host: cpal::platform::Host,
    // will receive data in constant intervall from distributor
    sender: Option<mpsc::Sender<CaptureEvent>>,
    stream: Option<cpal::Stream>,
}
impl Capture {
    pub fn new() -> Self{
        let host = cpal::default_host();

        return Self {
            channel_count: None,
            host,
            sender: None,
            stream: None,
        }
    }
    pub fn init(&mut self, device: &Device) -> Result<(), Error> {
        let (sender, receiver) = mpsc::channel();

        let (channel_count, stream) = match stream_audio_to_distributor(&self.host, sender.clone(), device) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        // initiates event handler
        thread::spawn(move || {
            handle_events(receiver);
        });

        self.channel_count = Some(channel_count);
        self.stream = Some(stream);
        self.sender = Some(sender);

        Ok(())
    }

    /// request a receiver that receives the distributed audio data as f32 samples
    ///
    /// you can request multiple receivers out of one Capture
    #[allow(unused_must_use)]
    pub fn get_receiver(&self) -> Option<CaptureReceiver> {
        if let Some(sender) = self.sender.clone() {
            Some(CaptureReceiver {sender})
        } else {
            None
        }
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

#[allow(unused_must_use)]
fn handle_events(receiver: mpsc::Receiver<CaptureEvent>) {
    let mut data: Vec<f32> = Vec::new();

    loop {
        if let Ok(event) = receiver.recv() {
            match event {
                CaptureEvent::SendData(mut d) => {
                    data.append(&mut d);
                }
                CaptureEvent::ReceiveData(sender) => {
                    //sender.send(data.clone());
                    if !data.is_empty() {
                        sender.send(Some(data.clone()));
                    } else {
                        sender.send(None);
                    }
                    data.drain(..);
                }
            }
        }
    }
}

fn stream_audio_to_distributor(
    host: &cpal::platform::Host,
    sender: mpsc::Sender<CaptureEvent>,
    device: &Device,
) -> Result<(u16, cpal::Stream), Error> {
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

    #[allow(unused_must_use)]
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                sender.send(CaptureEvent::SendData(data.to_vec()));
            },
            |e| warn!("error occurred on capture-stream: {}", e),
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| {
                let data = converter::i16_to_f32(data);
                sender.send(CaptureEvent::SendData(data.to_vec()));
            },
            |e| warn!("error occurred on capture-stream: {}", e),
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _: &_| {
                let data = converter::u16_to_f32(data);
                sender.send(CaptureEvent::SendData(data.to_vec()));
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

    Ok((channel_count, stream))
}
