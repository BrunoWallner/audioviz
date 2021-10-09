use std::thread;
use std::sync::mpsc;
use crate::processing::*;
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
                        while buffer.len() > config.resolution {
                            let c_b = 
                                convert_buffer(
                                    &buffer[0..config.resolution].to_vec(),
                                    config,
                                );
                            
                            calculated_buffer = if !calculated_buffer.is_empty() {
                                merge_buffers(&vec![calculated_buffer, c_b])
                            } else {
                                c_b
                            };

                            // remove already calculated parts
                            buffer.drain(0..config.resolution);
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
                        sender.send(config).unwrap();
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

    // modifying the amount of bars during runtime will result in unexpected behavior
    // unless sending 'Event::ClearBuffer' before
    // because the converter assumes that the bar amount stays the same
    // could be fixed by modifying ./src/processing/combine_buffers
    pub fn set_config(&self, config: Config) {
        self.event_sender.send(Event::SendConfig(config)).unwrap();
    }
    pub fn get_config(&self) -> Config {
        let (tx, rx) = mpsc::channel();
        self.event_sender.send(Event::RequestConfig(tx)).unwrap();
        rx.recv().unwrap()
    }
}