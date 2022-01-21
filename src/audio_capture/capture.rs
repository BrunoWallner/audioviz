//! Captures audio from system
//! 
//! then sends the data to the distributor which distributes one big buffer into multiple smaller ones
//! 
//! this increases overall smoothness at the cost of increased latency
//! 
//! On linux it can happen, that alsa prints to stderr
//! for this I recommend to use `https://github.com/Stebalien/gag-rs`

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::audio_capture::config::Config;

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
    ReceiveData(mpsc::Sender<Vec<f32>>),
}

pub struct CaptureReceiver {
    sender: mpsc::Sender<CaptureEvent>
}
impl CaptureReceiver {
    #[allow(unused_must_use)]
    pub fn receive_data(&self) -> Option<Vec<f32>> {
        let (sender, receiver) = mpsc::channel();
        self.sender.send(CaptureEvent::ReceiveData(sender));
        match receiver.recv() {
            Ok(val) => {
                Some(val)
            }
            Err(_) => None
        }
    }
}

pub struct Capture {
    // will receive data in constant intervall from distributor
    sender: mpsc::Sender<CaptureEvent>,
    _stream: cpal::Stream,
}
impl Capture {
    pub fn init(config: Config) -> Result<Self, Error> {
        let (sender, receiver) = mpsc::channel();

        let stream = match stream_audio_to_distributor(sender.clone(), config.clone()) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        // initiates event handler
        thread::spawn(move || {
            handle_events(receiver);
        });

        Ok(Self {
            sender,
            _stream: stream,
        })
    }

    /// request a receiver that receives the distributed audio data as f32 samples
    /// 
    /// you can request multiple receivers out of one Capture
    #[allow(unused_must_use)]
    pub fn get_receiver(&self) -> Result<CaptureReceiver, ()> {
        let sender = self.sender.clone();
        Ok(CaptureReceiver{sender})
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
    let mut sample_rate: f32 = 0.0; // in 1 / ms
    let mut last_send = Instant::now();
    let mut last_request = Instant::now(); // in ms

    let mut data: Vec<f32> = Vec::new();

    loop {
        if let Ok(event) = receiver.recv() {
            match event {
                CaptureEvent::SendData(mut d) => {
                    // calcs sample_rate
                    let elapsed: u128 = last_send.elapsed().as_nanos();
                    last_send = Instant::now();

                    sample_rate = d.len() as f32 / elapsed as f32;

                    data.append(&mut d);
                }
                CaptureEvent::ReceiveData(sender) => {
                    let elapsed: u128 = last_request.elapsed().as_nanos(); // time in µs
                    last_request = Instant::now();
                    
                    // approximation of what buffersize that gets sent and deleted
                    // results in smooth and continous output, replacement of prevoius Distributor

                    // time (in s) = sample_rate (in hz) * buf_size / : sample_rate
                    // time / sample_rate = buf_size
                    //
                    // time ms
                    // ---------- = buf_size
                    // sm_r 1/ms

                    let send_amount: usize = ( elapsed as f32 * sample_rate ) as usize + 1;
                    println!("send_amount: {}\nbuf_size: {}", send_amount, data.len());

                    if data.len() > send_amount {
                        let d = data[0..send_amount].to_vec();
                        data.drain(0..send_amount);

                        sender.send(d);
                    } else {
                        sender.send(data.clone());
                    }
                    
                }
            }
        }
    }
}

fn stream_audio_to_distributor(
    sender: mpsc::Sender<CaptureEvent>,
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
        move |data: &[f32], _: &_| { sender.send(CaptureEvent::SendData(data.to_vec())); },
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

/* 
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

    time (in s) = send_freq (in hz) * buf_size / : send_freq

    time / send_freq = buf_size

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
                println!("distributor")
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
*/
