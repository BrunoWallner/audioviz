use macroquad::prelude::*;

use audioviz::audio_capture::capture::{Capture, Device};
use audioviz::distributor::Distributor;
use audioviz::utils::{seperate_channels, apodize};

use audioviz::processor::{Processor, Plugin, Bandpass};

use std::io::Write;

const BUFFER_LENGTH: usize = 1024;
const BANDPASS: bool = true;

#[macroquad::main("AudioScope")]
async fn main() {
    let mut audio_capture = Capture::new();

    // device selection
    let devices = audio_capture.fetch_devices().unwrap();
    for (id, device) in devices.iter().enumerate() {
        println!("{id}\t{device}");
    }
    let id: usize = input("id: ").parse().unwrap_or(0);

    audio_capture.init(&Device::Id(id)).unwrap();
    let audio_receiver = audio_capture.get_receiver().unwrap();

    let mut distributor: Distributor<f32> = Distributor::new(44_100.0, Some(5000));

    let mut buffer: Vec<f32> = Vec::new();

    loop {
        if let Some(data) = audio_receiver.receive_data() {
            distributor.push_auto(&data);
        }
        let data = distributor.pop_auto(None);
        let data = seperate_channels(&data, audio_capture.channel_count.unwrap() as usize);
        let mut data: Vec<f32> = if !data.is_empty() {
            data[0].clone()
        } else {
            vec![]
        };
        buffer.append(&mut data);

        let wanted_buf_size: u64 = BUFFER_LENGTH as u64; 
        let drain_amount: isize = buffer.len() as isize - wanted_buf_size as isize;
        if drain_amount < buffer.len() as isize && drain_amount > 0 {
            buffer.drain(0..drain_amount as usize);
        }
        let mut data = buffer.clone();
        if !data.is_empty() {apodize(&mut data)}

        // bandpass-filter
        if BANDPASS && !data.is_empty() {
            let mut processor = Processor {
                data: data.to_vec(),
                sampling_rate: audio_capture.sampling_rate.unwrap() as f32,
                plugins: vec![
                    Plugin::Bandpass(Bandpass::new(100.0, 200.0, 5000.0, 6000.0)),
                ],
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
                    None => break
                };
                let y2: f32 = match data.peek() {
                    Some(d) => **d,
                    None => break
                };
                let y1: f32 = height / 2.0 - (y1 * height) + 1.0;
                let y2: f32 = height / 2.0 - (y2 * height) + 1.0;
    
                let x1: f32 = (x / buffer.len() as f32) * width;
                let x2: f32 = ( (x + 1.0) / buffer.len() as f32 ) * width;
    
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

    std::io::stdin().read_line(&mut input)
        .ok()
        .expect("Couldn't read line");
        
    input.trim().to_string()
}

