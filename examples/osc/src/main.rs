use audioviz::io::{Input, InputController};
use audioviz::io::Device;
use audioviz::utils::seperate_channels;
use audioviz::processor::{MultiChannelProcessor, Plugin, Lowpass, Highpass};

use macroquad::prelude::*;

const BUFFER_CAP: usize = 4096;

// shape of circle
const OFFSET_PERCENTAGE: f32 = 0.05;

const OVERLAP_PERCENTAGE: f32 = 0.0;

struct Data {
    input_controller: InputController,
    l_buffer: Vec<f32>,
    r_buffer: Vec<f32>,
    channel_count: usize,
}
impl Data {
    fn new(input_controller: InputController, channel_count: usize) -> Self {
        Self {
            input_controller,
            l_buffer: Vec::new(),
            r_buffer: Vec::new(),
            channel_count,
        }
    }

    fn get(&mut self) -> Vec<[f32; 2]> {
        assert_eq!(self.l_buffer.len(), self.r_buffer.len());

        // clear buffer
        let end: usize = self.l_buffer.len() - (self.l_buffer.len() as f32 * OVERLAP_PERCENTAGE) as usize;
        self.l_buffer.drain(0..end);
        self.r_buffer.drain(0..end);

        // get new data
        if let Some(data) = self.input_controller.pull_data() {
            // cap
            if self.l_buffer.len() > BUFFER_CAP * self.channel_count {
                let end = self.l_buffer.len() - BUFFER_CAP;
                self.l_buffer.drain(0..end);
                self.r_buffer.drain(0..end);
            }
            let out = if self.channel_count >= 2 {
                seperate_channels(&data, self.channel_count)
            } else {
                vec![
                    data.clone(),
                    data.clone()
                ]
            };

            self.l_buffer.append(&mut out[0].clone());
            self.r_buffer.append(&mut out[1].clone());
        }

        let mut out = [
            self.l_buffer.clone(),
            self.r_buffer.clone(),
        ];

        // apply offset
        let offset = (OFFSET_PERCENTAGE * out[0].len() as f32).floor() as usize;
        for _ in 0..offset {
            if !out[0].is_empty() {
                out[0].remove(0);
                out[1].pop();
            }
        }

        let mut combined: Vec<[f32; 2]> = Vec::new();
        for i in 0..out[0].len() {
            combined.push([
                out[0][i],
                out[1][i],
            ]);
        }

        combined
    }
}

#[macroquad::main("OSC")]
async fn main() {
    let mut audio_input = Input::new();

    // device selection
    let devices = audio_input.fetch_devices().unwrap();
    for (id, device) in devices.iter().enumerate() {
        println!("{id}\t{device}");
    }
    let id: usize = input("id: ").parse().unwrap_or(0);
    let (channel_count, sampling_rate, input_controller) = audio_input.init(&Device::Id(id)).unwrap();

    let mut data = Data::new(input_controller, channel_count as usize);
    loop {
        let height = screen_height();
        let width = screen_width();
        let thickness: f32 = (height + width) / 1000.0;
        let l_r = data.get();

        // calc points out of left and right channel
        let mut alpha: f32 = 0.0;
        let mut points: Vec<[f32; 2]> = Vec::new();
        for l_r in l_r.iter() {
            let p1 = l_r[0];
            let p2 = l_r[1];
            let a = (((p1 - 0.0).powi(2) + (p2 - 0.0).powi(2)).sqrt() * 255.0).clamp(0.0, 255.0);
            alpha += a;
            points.push([p1 * 0.9 + 0.5, p2 * -0.9 + 0.5]);
        }
        alpha /= l_r.len() as f32;
        let alpha = alpha.clamp(1.0, 255.0) as u8;

        // actual drawing using macroquad
        // lines
        let mut line_points = points.iter().peekable();
        loop {
            if let Some(p0) =  line_points.next() {
                if let Some(p1) = line_points.peek() {
                    draw_line(
                        p0[0] * width,
                        p0[1] * height,
                        p1[0] * width,
                        p1[1] * height,
                        thickness,
                        Color::from_rgba(255 - alpha, alpha, 100 - alpha, alpha.clamp(100, 255))
                    )
                } else {
                    break
                }
            } else {
                break
            }
        }

        // points
        for point in points.iter() {
            draw_circle(point[0] * width, point[1] * height, thickness * 1.5, Color::from_rgba(100 - alpha, alpha, 255 - alpha, alpha.clamp(100, 255)));
        }

        // -------------
        //    Treble 
        // -------------
        let mut data: Vec<f32> = Vec::new();
        for d in l_r.iter() {
            data.push(d[0]);
            data.push(d[1]);
        }
        let mut processor = MultiChannelProcessor {
            data,
            sampling_rate: sampling_rate as f32,
            plugins: vec![
                Plugin::Highpass(Highpass::new(5000.0, 4000.0))
            ],
            channel_count: channel_count as usize,
        };
        processor.process();
        let out = seperate_channels(&processor.data, channel_count as usize);
        let mut l_r_t: Vec<[f32; 2]> = Vec::new();
        for i in 0..out[0].len() {
            l_r_t.push([
                out[0][i],
                out[1][i],
            ]);
        }

        let mut points: Vec<[f32; 2]> = Vec::new();
        for l_r_t in l_r_t.iter() {
            let p1 = l_r_t[0];
            let p2 = l_r_t[1];
            points.push([p1 * 0.9 + 0.5, p2 * -0.9 + 0.5]);
        }
        let mut line_points = points.iter().peekable();
        loop {
            if let Some(p0) =  line_points.next() {
                if let Some(p1) = line_points.peek() {
                    draw_line(
                        p0[0] * width,
                        p0[1] * height,
                        p1[0] * width,
                        p1[1] * height,
                        thickness,
                        Color::from_rgba(0, 255, 55, 50)
                    )
                } else {
                    break
                }
            } else {
                break
            }
        }

        // -------------
        //     BASS
        // -------------
        let mut data: Vec<f32> = Vec::new();
        for d in l_r.iter() {
            data.push(d[0]);
            data.push(d[1]);
        }
        let mut processor = MultiChannelProcessor {
            data,
            sampling_rate: sampling_rate as f32,
            plugins: vec![
                Plugin::Lowpass(Lowpass::new(300.0, 500.0))
            ],
            channel_count: channel_count as usize,
        };
        processor.process();
        let out = seperate_channels(&processor.data, channel_count as usize);
        let mut l_r_b: Vec<[f32; 2]> = Vec::new();
        for i in 0..out[0].len() {
            l_r_b.push([
                out[0][i],
                out[1][i],
            ]);
        }

        let mut points: Vec<[f32; 2]> = Vec::new();
        for l_r_b in l_r_b.iter() {
            let p1 = l_r_b[0];
            let p2 = l_r_b[1];
            points.push([p1 * 0.9 + 0.5, p2 * -0.9 + 0.5]);
        }
        // points
        for point in points.iter() {
            draw_circle(point[0] * width, point[1] * height, thickness * 1.5, Color::from_rgba(0, 150, 255, 100));
        }

        next_frame().await
    }
}

use std::io::Write;

fn input(print: &str) -> String {
    print!("{}", print);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();

    std::io::stdin().read_line(&mut input)
        .ok()
        .expect("Couldn't read line");
        
    input.trim().to_string()
}
