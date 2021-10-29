use std::thread;
use std::sync::mpsc;
use crate::audio_data::*;
use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Event {
    RequestData(mpsc::Sender<Vec<f32>>),
    SendData(Vec<f32>),
    RequestConfig(mpsc::Sender<Config>),
    SendConfig(Config),
    RequestRefresh,
    ClearBuffer,
}

pub struct AudioStream {
    event_sender: mpsc::Sender<Event>,
}
impl AudioStream {
    pub fn init(config: Config) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();

        // thread that receives Events, converts and processes the received data 
        // and sends it via a mpsc channel to requesting to thread that requested processed data
        thread::spawn(move || {
            //let (event_sender, event_receiver) = mpsc::channel();
            let mut buffer: Vec<f32> = Vec::new();
            let mut calculated_buffer: Vec<f32> = Vec::new();
            let mut smoothing_buffer: Vec<Vec<f32>> = Vec::new();
            let mut smoothed_buffer: Vec<f32> = Vec::new();
            let mut config: Config = config;
    
            loop  {
                match event_receiver.recv().unwrap() {
                    Event::SendData(mut b) => {
                        buffer.append(&mut b);
                        let fft_res = config.fft_resolution;
                        while buffer.len() > fft_res {
                            let mut audio_data = AudioData::new(config.clone(), &buffer[0..fft_res].to_vec());
                            audio_data.fft();
                            audio_data.distribute_volume();
                            audio_data.cut_off();
                            audio_data.normalize();
                            audio_data.smooth();
                            audio_data.apply_resolution();

                            let c_b = audio_data.buffer;

                            /*
                            let c_b = 
                                convert_buffer(
                                    &buffer[0..fft_res].to_vec(),
                                    &config,
                                );
                            */
                            
                            calculated_buffer = if !calculated_buffer.is_empty() {
                                merge_buffers(&vec![calculated_buffer, c_b])
                            } else {
                                c_b
                            };

                            // remove already calculated parts
                            //buffer.drain(0..config.fft_resolution);
                            buffer.drain(0..config.fft_resolution / 2); // overlapping
                        }
                    },
                    Event::RequestData(sender) => {
                        sender.send(smoothed_buffer.clone()).expect("audio thread lost connection to bridge");
                    }
                    Event::RequestRefresh => {
                        if !calculated_buffer.is_empty() {
                            smoothing_buffer.push(calculated_buffer.clone());
                        }
                        smoothed_buffer = if !smoothing_buffer.is_empty() {
                            merge_buffers(&smoothing_buffer)
                        } else {
                            Vec::new()
                        };
                        while smoothing_buffer.len() > config.buffering {
                            smoothing_buffer.remove(0);
                        }
                    }
                    Event::RequestConfig(sender) => {
                        sender.send(config.clone()).unwrap();
                    }
                    Event::SendConfig(c) => {
                        config = c;
                    }
                    Event::ClearBuffer => {
                        calculated_buffer = Vec::new();
                    }
                }
            }
        });
        let event_sender_clone = event_sender.clone();
        thread::spawn(move || loop {
            // receiving refresh rate from main thread
            let (tx, rx) = mpsc::channel();
            event_sender_clone.send(Event::RequestConfig(tx)).unwrap();
            let config = rx.recv().unwrap();

            thread::sleep(std::time::Duration::from_millis(1000 / config.refresh_rate as u64));
            event_sender_clone.send(Event::RequestRefresh).unwrap();
        });
    
        AudioStream {
            event_sender,
        }
    }
    pub fn get_audio_data(&self) -> Vec<f32> {
        let (tx, rx) = mpsc::channel();
        self.event_sender.send(Event::RequestData(tx)).unwrap();
        rx.recv().unwrap()
    }
    pub fn get_event_sender(&self) -> mpsc::Sender<Event> {
        self.event_sender.clone()
    }

    pub fn adjust_volume(&self, v: f32) {
        let config = self.get_config();
        let config = Config {
            volume: config.volume * v,
            ..config
        };
        self.set_config(config);
    }

    // modifying the amount of bars during runtime will result in unexpected behavior
    // unless sending 'Event::ClearBuffer' before
    // because the converter assumes that the bar amount stays the same
    // could be fixed by modifying ./src/processing/combine_buffers
    pub fn set_config(&self, config: Config) {
        self.event_sender.send(Event::SendConfig(config)).unwrap();
    
    }

    pub fn set_bar_number(&self, number: usize) {
        let config = self.get_config();
        let current_bars: f32 = config.fft_resolution as f32 * 0.25 * (config.max_frequency as f32 / 20_000.0);
        let wanted_res: f32 = number as f32 / current_bars;

        let wanted_conf = Config {
            resolution: wanted_res,
            ..config
        };

        self.event_sender.send(Event::SendConfig(wanted_conf)).unwrap();
        self.event_sender.send(Event::ClearBuffer).unwrap();

    }

    pub fn get_config(&self) -> Config {
        let (tx, rx) = mpsc::channel();
        self.event_sender.send(Event::RequestConfig(tx)).unwrap();
        rx.recv().unwrap()
    }
}