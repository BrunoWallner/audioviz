use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::thread;

use crate::audio_capture::config::Config;

#[derive(Clone, Debug)]
pub enum Error {
    DeviceNotFound,
    DeviceNotAvailable,
    UnsupportedConfig,
    BackendSpecific(String),
}

// POV of event sender
#[derive(Clone, Debug)]
enum CaptureEvent {
    RequestReceiver(mpsc::Sender<mpsc::Receiver<CaptureEvent>>),
    SendData(Vec<f32>),
    ReceiveData(Vec<f32>),
}

pub struct CaptureReceiver {
    receiver: mpsc::Receiver<CaptureEvent>,
}
impl CaptureReceiver {
    pub fn receive_data(&self) -> Result<Vec<f32>, ()> {
        match self.receiver.recv() {
            Ok(event) => match event {
                CaptureEvent::ReceiveData(d) => Ok(d),
                _ => Err(())
            }
            Err(_) => Err(())
        }
    }
}

pub struct Capture {
    // will receive data in constant intervall from distributor
    //pub receiver: mpsc::Receiver<Vec<f32>>,
    sender: mpsc::Sender<CaptureEvent>,
    _stream: cpal::Stream,
}
impl Capture {
    pub fn init(config: Config) -> Result<Self, Error> {
        let (sender, receiver) = mpsc::channel();

        // bridge between distributor and sender
        let (dis_sender, dis_receiver) = mpsc::channel();

        let stream = match stream_audio_to_distributor(dis_sender.clone(), config.clone()) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        // initiates distributor
        let sender_clone = sender.clone();
        thread::spawn(move || init_distributor(dis_receiver, dis_sender, sender_clone, config));

        // initiates event handler
        thread::spawn(move || {
            handle_events(receiver);
        });

        Ok(Self {
            sender,
            _stream: stream,
        })
    }

    #[allow(unused_must_use)]
    /*
    pub fn request_receiver(&self) -> Result<mpsc::Receiver<CaptureEvent>, ()> {
        let (sender, receiver) = mpsc::channel();
        self.sender.send(CaptureEvent::RequestReceiver(sender));
        match receiver.recv() {
            Ok(r) => Ok(r),
            Err(_) => Err(())
        }
    }
    */
    pub fn get_receiver(&self) -> Result<CaptureReceiver, ()> {
        let (sender, receiver) = mpsc::channel();
        self.sender.send(CaptureEvent::RequestReceiver(sender));
        let receiver = match receiver.recv() {
            Ok(r) => r,
            Err(_) => return Err(())
        };
        Ok(CaptureReceiver{receiver})
    }

    pub fn fetch_devices() -> Result<Vec<String>, Error> {
        let host = cpal::default_host();
        let devices = match host.devices() {
            Ok(d) => d,
            Err(e) => match e {
                cpal::DevicesError::BackendSpecific { err } => {
                    let cpal::BackendSpecificError { description } = err;
                    return Err(Error::BackendSpecific(description))
                }
            }
        };
        let devices: Vec<String> = devices.into_iter()
        .map(
            |dev| dev.name().unwrap_or_else(|_| String::from("invalid")
        ))
        .collect();

        Ok(devices)
    }
}

#[allow(unused_must_use)]
fn handle_events(
    receiver: mpsc::Receiver<CaptureEvent>,
) {
    let mut sender: Vec<mpsc::Sender<CaptureEvent>> = Vec::new();
    loop {
        if let Ok(event) = receiver.recv() {
            match event {
                CaptureEvent::SendData(data) => {
                    if !sender.is_empty() {
                        for sender in sender.iter() {
                            sender.send(CaptureEvent::ReceiveData(data.clone()));
                        }
                    }
                }
                CaptureEvent::RequestReceiver(outer_sender) => {
                    let (sen, recv) = mpsc::channel();
                    sender.push(sen);
                    outer_sender.send(recv);
                }
                CaptureEvent::ReceiveData(_) => { /* should not be sent */ }
            }
        }
    }
}

fn stream_audio_to_distributor(
    sender: mpsc::Sender<DistributorEvent>,
    config: Config,
) -> Result<cpal::Stream, Error> {
    let host = cpal::default_host();

    let device = match config.device.as_str() {
        "default" => match host.default_input_device() {
            Some(d) => d,
            None => return Err(Error::DeviceNotFound),
        },
        device => match host.input_devices() {
            Ok(mut devices) => {
                match devices.find(|x| x.name().map(|y| y == *device).unwrap_or(false)) {
                    Some(d) => d,
                    None => return Err(Error::DeviceNotFound),
                }
            }
            Err(_) => return Err(Error::DeviceNotFound),
        },
    };

    let device_config = cpal::StreamConfig {
        channels: 1,
        sample_rate: match config.sample_rate {
            Some(rate) => cpal::SampleRate(rate),
            None => cpal::SampleRate(44_100),
        },
        buffer_size: match config.latency {
            Some(latency) => cpal::BufferSize::Fixed(latency),
            None => cpal::BufferSize::Default,
        },
    };

    #[allow(unused_must_use)]
    let stream = match device.build_input_stream(
        &device_config,
        move |data: &[f32], _: &_| { sender.send(DistributorEvent::IncomingData(data.to_vec())); },
        |_| (),
    ) {
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

    Ok(stream)
}

enum DistributorEvent {
    IncomingData(Vec<f32>),
    BufferPushRequest,
}

// converts choppy buffers received from cpal to more continous buffers
fn init_distributor(
    receiver: mpsc::Receiver<DistributorEvent>,
    distributor_event_sender: mpsc::Sender<DistributorEvent>,
    sender: mpsc::Sender<CaptureEvent>,
    config: Config,
) {
    let sample_rate: u32 = config.sample_rate.unwrap_or(44_100);
    let micros_to_wait: u64 = 1_000_000 / sample_rate as u64 * config.buffer_size as u64;

    // reduces risk of buffer growing
    let micros_to_wait = (micros_to_wait as f32 * 0.95) as u64;

    let mut buffer: Vec<f32> = Vec::new();
    thread::spawn(move || loop {
        if let Ok(event) = receiver.recv() { match event {
            DistributorEvent::IncomingData(mut data) => {
                buffer.append(&mut data);
            }
            DistributorEvent::BufferPushRequest => {
                if buffer.len() > config.buffer_size as usize {
                    sender
                        .send(CaptureEvent::SendData(
                            buffer[0..=config.buffer_size as usize].to_vec())
                        )
                        .ok();

                    // clears already pushed parts
                    buffer.drain(0..=config.buffer_size as usize);
                }
                if buffer.len() > config.max_buffer_size as usize {
                    let diff: usize = buffer.len() - config.max_buffer_size as usize;
                    buffer.drain(..diff);
                }
            }
        }
    }});

    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_micros(micros_to_wait));
        distributor_event_sender
            .send(DistributorEvent::BufferPushRequest)
            .ok();
    });
}
