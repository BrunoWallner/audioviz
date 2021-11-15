use crate::audio_data::*;
use crate::config::Config;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub enum Event {
    RequestData(mpsc::Sender<Vec<f32>>),
    SendData(Vec<f32>),
    RequestConfig(mpsc::Sender<Config>),
    SendConfig(Config),
    RequestRefresh,
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
            let mut buffer: Vec<f32> = Vec::new();
            let mut calculated_buffer: Vec<Vec<f32>> = Vec::new();
            let mut current_buffer: Vec<f32> = Vec::new();

            let mut gravity_buffer: Vec<f32> = Vec::new();
            let mut gravity_time_buffer: Vec<u32> = Vec::new();

            let mut config: Config = config;

            loop {
                match event_receiver.recv().unwrap() {
                    Event::SendData(mut b) => {
                        buffer.append(&mut b);
                        let fft_res: usize = config.fft_resolution;
                        while buffer.len() > fft_res {
                            let mut audio_data =
                                AudioData::new(config.clone(), &buffer[0..fft_res].to_vec());
                            audio_data.compute_all();

                            calculated_buffer.insert(0, audio_data.buffer);

                            // remove already calculated parts
                            let cutoff: f32 = match config.pre_fft_buffer_cutoff {
                                d if (0.001..=1.0).contains(&d) => d,
                                _ => 0.5,
                            };
                            buffer.drain(0..(fft_res as f32 * cutoff) as usize);
                            // overlapping
                        }
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
                        //println!("buf_len: {}", calculated_buffer.len());
                        if !calculated_buffer.is_empty() {
                            current_buffer = calculated_buffer.pop().unwrap();
                            
                        }

                        /* Gravity */
                        match config.gravity {
                            Some(gravity) => {
                                if gravity_buffer.len() != current_buffer.len() {
                                    gravity_buffer = vec![0.0; current_buffer.len()];
                                }
                                if gravity_time_buffer.len() != current_buffer.len() {
                                    gravity_time_buffer = vec![0; current_buffer.len()];
                                }

                                // applies up velocity
                                for i in 0..current_buffer.len() {
                                    if gravity_buffer[i] < current_buffer[i] {
                                        gravity_buffer[i] = current_buffer[i];
                                        gravity_time_buffer[i] = 0;
                                    } else {
                                        gravity_time_buffer[i] += 1;
                                    }
                                }

                                // apply gravity to buffer
                                for (i, v) in gravity_buffer.iter_mut().enumerate() {
                                    *v -= gravity * 0.0025 * (gravity_time_buffer[i] as f32);
                                }
                            }
                            None => (),
                        }
                    }
                    Event::RequestConfig(sender) => {
                        sender.send(config.clone()).unwrap();
                    }
                    Event::SendConfig(c) => {
                        config = c;
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
