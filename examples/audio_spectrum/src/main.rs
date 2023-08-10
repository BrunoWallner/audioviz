use macroquad::prelude::*;

use audioviz::io::{Device, Input};
use audioviz::spectrum::{
    config::{Interpolation, ProcessorConfig, StreamConfig},
    stream::Stream,
    Frequency,
};

use std::io::Write;

#[macroquad::main("AudioSpectrum")]
async fn main() {
    let mut audio_input = Input::new();

    // device selection
    let devices = audio_input.fetch_devices().unwrap();
    for (id, device) in devices.iter().enumerate() {
        println!("{id}\t{device}");
    }
    let id: usize = input("id: ").parse().unwrap_or(0);

    let (channel_count, _sampling_rate, audio_receiver) =
        audio_input.init(&Device::Id(id), Some(1024)).unwrap();

    let stream_config: StreamConfig = StreamConfig {
        channel_count,
        gravity: Some(2.0),
        fft_resolution: 1024 * 3,
        processor: ProcessorConfig {
            frequency_bounds: [35, 20_000],
            interpolation: Interpolation::Cubic,
            volume: 0.4,
            resolution: None,
            ..ProcessorConfig::default()
        },
        ..StreamConfig::default()
    };
    let mut stream: Stream = Stream::new(stream_config);
    loop {
        if let Some(new_data) = audio_receiver.pull_data() {
            stream.push_data(new_data);
        }

        stream.update();

        let frequencies: Vec<Vec<Frequency>> = stream.get_frequencies();
        let frequencies: Vec<Frequency> = if frequencies.len() >= 2 {
            let mut buf: Vec<Frequency> = Vec::new();
            // left
            let mut left = frequencies[0].clone();
            left.reverse();
            buf.append(&mut left);
            // right
            buf.append(&mut frequencies[1].clone());
            buf
        } else {
            if frequencies.len() == 1 {
                frequencies[0].clone()
            } else {
                Vec::new()
            }
        };

        clear_background(BLACK);

        // draw lines
        let height = screen_height();
        let width = screen_width();

        let mut freqs = frequencies.iter().peekable();
        let mut x: f32 = 0.5;

        loop {
            // determines positions of line
            let f1: &Frequency = match freqs.next() {
                Some(d) => d,
                None => break,
            };
            let f2: &Frequency = match freqs.peek() {
                Some(d) => *d,
                None => break,
            };
            let y1: f32 = height - (f1.volume * height);
            let y2: f32 = height - (f2.volume * height);

            let x1: f32 = (x / frequencies.len() as f32) * width;
            let x2: f32 = ((x + 1.0) / frequencies.len() as f32) * width;

            draw_line(x1, y1, x2, y2, 4.0, WHITE);

            x += 1.0;
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
