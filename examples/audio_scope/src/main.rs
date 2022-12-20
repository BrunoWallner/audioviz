use macroquad::prelude::*;

use audioviz::io::{Device, Input};
use audioviz::utils::{apodize, seperate_channels};

use audioviz::processor::{Bandpass, Plugin, Processor};

use std::io::Write;

const BUFFER_LENGTH: usize = 1024;
const BANDPASS: bool = false;

#[macroquad::main("AudioScope")]
async fn main() {
    let mut audio_input = Input::new();

    // device selection
    let devices = audio_input.fetch_devices().unwrap();
    for (id, device) in devices.iter().enumerate() {
        println!("{id}\t{device}");
    }
    let id: usize = input("id: ").parse().unwrap_or(0);

    let (channel_count, sampling_rate, audio_receiver) = audio_input.init(&Device::Id(id)).unwrap();

    let mut buffer: Vec<f32> = Vec::new();

    loop {
        if let Some(new_data) = audio_receiver.pull_data() {
            let mut data = seperate_channels(&new_data, channel_count as usize);
            buffer.append(&mut data[0]);
        }

        let wanted_buf_size: u64 = BUFFER_LENGTH as u64;
        let drain_amount: isize = buffer.len() as isize - wanted_buf_size as isize;
        if drain_amount < buffer.len() as isize && drain_amount > 0 {
            buffer.drain(0..drain_amount as usize);
        }
        let mut data = buffer.clone();
        if !data.is_empty() {
            apodize(&mut data);
        }

        // bandpass-filter
        if BANDPASS && !data.is_empty() {
            let mut processor = Processor {
                data: data.to_vec(),
                sampling_rate: sampling_rate as f32,
                plugins: vec![Plugin::Bandpass(Bandpass::new(
                    100.0, 200.0, 5000.0, 6000.0,
                ))],
            };
            processor.process();
            data = processor.data;
        }

        clear_background(BLACK);

        // draw lines
        let height = screen_height();
        let width = screen_width();

        if !data.is_empty() {
            let mut data = data.iter().peekable();
            let mut x: f32 = 0.5;
            loop {
                // determines positions of line
                let y1: f32 = match data.next() {
                    Some(d) => *d,
                    None => break,
                };
                let y2: f32 = match data.peek() {
                    Some(d) => **d,
                    None => break,
                };
                let y1: f32 = height / 2.0 - (y1 * height) + 1.0;
                let y2: f32 = height / 2.0 - (y2 * height) + 1.0;

                let x1: f32 = (x / buffer.len() as f32) * width;
                let x2: f32 = ((x + 1.0) / buffer.len() as f32) * width;

                draw_line(x1, y1, x2, y2, 1.0, WHITE);

                x += 1.0;
            }
        }
        next_frame().await
    }
}

fn input(print: &str) -> String {
    print!("{}", print);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .ok()
        .expect("Couldn't read line");

    input.trim().to_string()
}
