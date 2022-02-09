use macroquad::prelude::*;

use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
use audioviz::distributor::{Distributor, Elapsed};
use std::time::Instant;

const SAMPLE_RATE: u64 = 44_100; // in hz
const BUFFER_DURATION: u64 = 90000; // in µs

#[macroquad::main("AudioScope")]
async fn main() {
    let config = CaptureConfig {
        sample_rate: None,
        ..Default::default()
    };
    let audio_capture = Capture::init(config).unwrap();
    let audio_receiver = audio_capture.get_receiver().unwrap();

    let mut distributor: Distributor<f32> = Distributor::new(44_100.0, 5000);

    let mut buffer: Vec<f32> = Vec::new();

    // neccessary for distributor
    let mut delta_push: Instant = Instant::now();
    let mut delta_pop: Instant = Instant::now();

    loop {
        if let Some(data) = audio_receiver.receive_data() {
            let elapsed = delta_push.elapsed().as_micros();
            distributor.push(&data, Elapsed::Micros(elapsed));
            delta_push = Instant::now();
        }
        let elapsed = delta_pop.elapsed().as_micros();
        let mut data = distributor.pop(Elapsed::Micros(elapsed));
        delta_pop = Instant::now();
        buffer.append(&mut data);

        let wanted_buf_size: u64 = ((1000.0 / SAMPLE_RATE as f32) * BUFFER_DURATION as f32) as u64; 
        let drain_amount: isize = buffer.len() as isize - wanted_buf_size as isize;
        if drain_amount < buffer.len() as isize && drain_amount > 0 {
            buffer.drain(0..drain_amount as usize);
        }

        clear_background(BLACK);
        
        // draw lines
        let height = screen_height();
        let width = screen_width();

        let mut data = buffer.iter().peekable();
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

            draw_line(x1, y1, x2, y2, 4.0, WHITE);
	    
            x += 1.0;
        }

        next_frame().await
    }
}
