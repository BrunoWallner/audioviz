use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::thread;

use crate::audio_capture::config::Config;

#[derive(Copy, Clone, Debug)]
pub enum Error {
    DeviceNotFound,
}

#[derive(Debug)]
pub struct Capture {
    // will receive data in constant intervall from distributor
    pub receiver: mpsc::Receiver<Vec<f32>>,
}
impl Capture {
    pub fn init(config: Config) -> Self {
        let (sender, receiver) = mpsc::channel();

        // bridge between distributor and sender
        let (dis_sender, dis_receiver) = mpsc::channel();

        // initiates thread that sends captured audio data to distributor
        let d_s = dis_sender.clone();
        let c = config.clone();
        thread::spawn(move || {
            let _stream = stream_audio_to_distributor(d_s, c);
            thread::park();
        });

        // initiates distributor
        thread::spawn(move || {
            init_distributor(dis_receiver, dis_sender, sender, config)
        });

        Self {
            receiver
        }
    }
}

fn stream_audio_to_distributor(
    sender: mpsc::Sender<DistributorEvent>,
    config: Config,
) -> Result<cpal::Stream, Error> {
    //let _print_gag = Gag::stderr().unwrap();
    let host = cpal::default_host();

    let device = match config.device.as_str() {
        "default" => match host.default_input_device() {
            Some(d) => d,
            None => return Err(Error::DeviceNotFound),
        },
        device => match host.input_devices() {
            Ok(mut devices) => {
                match devices.find(|x| x.name().map(|y| y == device.to_string()).unwrap_or(false)) {
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

    let stream = device
        .build_input_stream(
            &device_config.into(),
            move |data: &[f32], _: &_| {
                match sender.send(DistributorEvent::IncomingData(data.to_vec())) {
                    Ok(_) => (),
                    Err(_) => (),
                }
            },
            |_| (),
        )
        .unwrap();

    stream.play().unwrap();

    Ok(stream)
}

enum DistributorEvent {
    IncomingData(Vec<f32>),
    BufferPushRequest,
}

fn init_distributor(
    receiver: mpsc::Receiver<DistributorEvent>,
    distributor_event_sender: mpsc::Sender<DistributorEvent>,
    sender: mpsc::Sender<Vec<f32>>,
    config: Config,
) {
    let sample_rate: u32 = match config.sample_rate {
        Some(s) => s,
        None => 44_100,
    };
    let micros_to_wait: u64 = 1_000_000 / sample_rate as u64 * config.buffer_size as u64;

    let mut buffer: Vec<f32> = Vec::new();
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => match event {
                DistributorEvent::IncomingData(mut data) => {
                    buffer.append(&mut data);
                }
                DistributorEvent::BufferPushRequest => {
                    sender.send(buffer.clone()).ok();
                    // clears already pushed parts
                    if buffer.len() > config.buffer_size as usize * 1 {
                        buffer.drain(0..config.buffer_size as usize);
                    }
                }
            },
            Err(_) => (),
        }
    });

    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_micros(micros_to_wait));
        distributor_event_sender.send(DistributorEvent::BufferPushRequest).ok();
    });
}
