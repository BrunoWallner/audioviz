use crate::audio_data::*;
use crate::config::{Config, Interpolation};
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
                    Event::RequestData(sender) => {
                        let buf = match config.gravity {
                            Some(_) => {
                                gravity_buffer.clone()
                            }
                            None => {
                                current_buffer.clone()
                            }
                        };

                        // APPLIES POSITIONS TO FREQUENCIES and interpolation
                        // VERY IMPORTANT
                        let buf = match config.interpolation {
                            Interpolation::None => buf,
                            Interpolation::Gaps => {
                                let mut o_buf: Vec<Frequency> = vec![Frequency::empty(); buf.len()];
                                for freq in buf.iter() {
                                    let abs_pos = (o_buf.len() as f32 * freq.position) as usize;
                                    if o_buf.len() > abs_pos {

                                        // louder freqs are more important and shall not be overwritten by others
                                        if freq.volume > o_buf[abs_pos].volume {
                                            o_buf[abs_pos] = freq.clone();
                                        }
                                    }
                                }
                                o_buf
                            },
                            Interpolation::Step => {
                                let mut o_buf: Vec<Frequency> = vec![Frequency::empty(); buf.len()];
                                let mut freqs = buf.iter().peekable();
                                'filling: loop {
                                    let freq: &Frequency = match freqs.next() {
                                        Some(f) => f,
                                        None => break 'filling,
                                    };
                                    
                                    let start: usize = (freq.position * o_buf.len() as f32) as usize;
                                    let end = 
                                    (
                                        match freqs.peek() {
                                            Some(f) => f.position,
                                            None => 1.0,
                                        } * o_buf.len() as f32
                                    ) as usize;
                                    
                                    for i in start..end {
                                        if o_buf.len() > i {
                                            o_buf[i] = freq.clone();
                                        }
                                    }
                                }

                                o_buf
                            }
                            _ => buf
                        };

                        // freq bounds
                        let mut start: usize = 0;
                        let mut i: usize = 0;
                        loop {
                            if i >= buf.len() {
                                break;
                            }
                            if buf[i].freq > config.frequency_bounds[0] as f32 {
                                start = i;
                                break;
                            }
                            i += 1;
                        }

                        let mut end: usize = 0;
                        let mut i: usize = 0;
                        loop {
                            if i >= buf.len() {
                                break;
                            }
                            if buf[buf.len() - (i + 1)].freq < config.frequency_bounds[1] as f32 {
                                end = buf.len() - i;
                                break;
                            }
                            i += 1;
                        }

                        let buf = buf[start..end].to_vec();

                        // applies resolution
                        let o_buf = apply_bar_count(&buf, config.bar_count);

                        // finally sends buffer to request channel
                        sender.send(o_buf).unwrap();
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


#[allow(clippy::collapsible_if)]
fn apply_bar_count(buffer: &Vec<Frequency>, bar_count: usize) -> Vec<Frequency> {
    let current_bars: f32 = buffer.len() as f32;
    let resolution: f32 = bar_count as f32 / current_bars;

    let mut output_buffer: Vec<Frequency> =
        vec![Frequency::empty(); (buffer.len() as f32 * resolution) as usize];

    if resolution < 1.0 {
        let offset = output_buffer.len() as f32 / buffer.len() as f32;
        for (i, freq) in buffer.iter().enumerate() {
            let pos = (i as f32 * offset) as usize;

            // cannot be collapsed as clippy notes i think
            if pos < output_buffer.len() {
                // crambling type
                if output_buffer[pos].volume < freq.volume {
                    output_buffer[pos] = Frequency {volume: freq.volume, freq: freq.freq, position: freq.position};
                }
            }
        }

        output_buffer
    }
    else {
        Vec::new()
    }
}
