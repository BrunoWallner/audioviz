use crate::audio_data::*;
use crate::config::Config;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub enum Event {
    RequestData(mpsc::Sender<Vec<Frequency>>),
    SendData(Vec<f32>),
    RequestConfig(mpsc::Sender<Config>),
    SendConfig(Config),
    RequestRefresh,
}

enum ConverterEvent {
    RequestData(mpsc::Sender<Vec<Frequency>>),
    SendRawData(Vec<f32>),
    SendConfig(Config),
}

pub struct AudioStream {
    event_sender: mpsc::Sender<Event>,
}
impl AudioStream {
    pub fn init(config: Config) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();

        // thread that computes and converts Data
        let (converter_sender, converter_receiver) = mpsc::channel();
        let config_clone = config.clone();
        thread::spawn(move || {
            let mut buffer: Vec<f32> = Vec::new();
            let mut config: Config = config_clone;
            loop {
                match converter_receiver.recv().unwrap() {
                    ConverterEvent::RequestData(sender) => {
                        let fft_res: usize = config.fft_resolution;

                        if buffer.len() > fft_res {

                            // clears unimportant buffer values
                            let diff = buffer.len() - fft_res;
                            buffer.drain(..diff);

                            let mut audio_data = AudioData::new(
                                config.clone(),
                                &buffer[..].to_vec(),
                            );
                            audio_data.compute_all();
                            
                            // must only iterate ONCE
                            sender.send(audio_data.buffer).ok();
                        }
                    },
                    ConverterEvent::SendRawData(mut data) => {
                        buffer.append(&mut data);
                    }
                    ConverterEvent::SendConfig(conf) => {
                        config = conf;
                    }
                }
            }
        });

        // thread that receives Events
        // and sends it via a mpsc channel to requesting to thread that requested processed data
        //let event_sender_clone = event_sender.clone();
        thread::spawn(move || {
            //let mut buffer: Vec<f32> = Vec::new();
            let mut current_buffer: Vec<Frequency> = Vec::new();

            let mut gravity_buffer: Vec<Frequency> = Vec::new();
            let mut gravity_time_buffer: Vec<u32> = Vec::new();

            let mut config: Config = config;

            loop {
                match event_receiver.recv().unwrap() {
                    Event::SendData(b) => {
                        converter_sender.send(ConverterEvent::SendRawData(b)).unwrap();
                    }
                    Event::RequestData(sender) => match config.gravity {
                        Some(_) => {
                            sender
                                .send(gravity_buffer.clone())
                                .expect("audio thread lost connection to bridge");
                        }
                        None => {
                            sender
                                .send(current_buffer.clone())
                                .expect("audio thread lost connection to bridge");
                        }
                    },
                    Event::RequestRefresh => {
                        // request data from converter thread
                        let (tx, rx) = mpsc::channel();
                        converter_sender.send(ConverterEvent::RequestData(tx)).unwrap();
                        current_buffer = match rx.recv() {
                            Ok(buf) => buf,
                            Err(_) => current_buffer
                        };

                        /* Gravity */
                        match config.gravity {
                            Some(gravity) => {
                                if gravity_buffer.len() != current_buffer.len() {
                                    gravity_buffer = vec![Frequency::empty(); current_buffer.len()];
                                }
                                if gravity_time_buffer.len() != current_buffer.len() {
                                    gravity_time_buffer = vec![0; current_buffer.len()];
                                }

                                // sets value of gravity_buffer to current_buffer if current_buffer is higher
                                for i in 0..current_buffer.len() {
                                    if gravity_buffer[i].volume < current_buffer[i].volume {
                                        gravity_buffer[i] = current_buffer[i].clone();
                                        gravity_time_buffer[i] = 0;
                                    } else {
                                        gravity_time_buffer[i] += 1;
                                    }
                                }

                                // apply gravity to buffer
                                for (i, freq) in gravity_buffer.iter_mut().enumerate() {
                                    freq.volume -= gravity * 0.0025 * (gravity_time_buffer[i] as f32);
                                }
                            }
                            None => (),
                        }
                    }
                    Event::RequestConfig(sender) => {
                        sender.send(config.clone()).unwrap();
                    }
                    Event::SendConfig(c) => {
                        config = c.clone();
                        converter_sender.send(ConverterEvent::SendConfig(c)).unwrap();
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

            thread::sleep(std::time::Duration::from_millis(
                1000 / config.refresh_rate as u64,
            ));
            event_sender_clone.send(Event::RequestRefresh).unwrap();
        });

        AudioStream { event_sender }
    }
    pub fn get_audio_data(&self) -> Vec<Frequency> {
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

        let wanted_conf = Config {
            bar_count: number,
            ..config
        };

        self.event_sender
            .send(Event::SendConfig(wanted_conf))
            .unwrap();
    }

    pub fn get_config(&self) -> Config {
        let (tx, rx) = mpsc::channel();
        self.event_sender.send(Event::RequestConfig(tx)).unwrap();
        rx.recv().unwrap()
    }
}

// combines 2-dimensional buffer (Vec<Vec<f32>>) into a 1-dimensional one that has the average value of the 2D buffer
// EVERY 1D buffer of whole buffer MUST have the same length, but the current implementation guarantees this, considering the resolution stays the same
// if size changes you have to call 'Event::ClearBuffer'
/*
#[allow(clippy::ptr_arg)]
pub fn merge_buffers(
    buffer: &Vec<Vec<f32>>, // EVERY 1D buffer of whole buffer MUST have the same length
) -> Result<Vec<f32>, ()> {
    // checks if buffers are valid
    if buffer.len() == 0 {
        return Err(());
    }
    let buf_len: usize = buffer[0].len();
    for i in buffer.iter() {
        if i.len() != buf_len {
            return Err(());
        }
    }

    let mut smoothed_percentage: f32 = 0.0;
    let mut output_buffer: Vec<f32> = vec![0.0; buffer[0].len()];
    for (pos_z, z_buffer) in buffer.iter().enumerate() {
        // needed for weighting the Importance of earch z_buffer, more frequent -> more important
        // should decrease latency and increase overall responsiveness
        let percentage: f32 = (pos_z + 1) as f32 / buffer.len() as f32;
        smoothed_percentage += percentage;
        for (pos_x, value) in z_buffer.iter().enumerate() {
            if pos_x < output_buffer.len() {
                output_buffer[pos_x] += value * percentage;
            }
        }
    }

    for b in output_buffer.iter_mut() {
        *b /= smoothed_percentage;
    }

    Ok(output_buffer)
}
*/
