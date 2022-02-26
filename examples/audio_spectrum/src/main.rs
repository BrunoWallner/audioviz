use macroquad::prelude::*;

use audioviz::audio_capture::capture::{Capture, Device};
use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig, Interpolation}, stream::Stream};
use audioviz::distributor::Distributor;

use audioviz::processor::{Processor, Plugin};

use std::io::Write;

const LOWPASS: bool = false;

#[macroquad::main("AudioSpectrum")]
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
    let stream_config: StreamConfig = StreamConfig {
        channel_count: audio_capture.channel_count.unwrap(),
        gravity: Some(6.0),
        fft_resolution: 1024 * 4,
        processor: ProcessorConfig {
            frequency_bounds: [50, 20_000],
	    interpolation: Interpolation::Step,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut stream: Stream = Stream::new(stream_config);

    loop {
        if let Some(data) = audio_receiver.receive_data() {
            distributor.push_auto(&data);
        }
        let mut data = distributor.pop_auto(None);

        if !data.is_empty() {
            // lowpass-filter
            if LOWPASS {
                let mut processor = Processor {
                    data: data,
                    sampling_rate: audio_capture.sampling_rate.unwrap_or(0) as f32,
                    plugins: vec![
                        Plugin::Lowpass{cutoff_frequency: 100.0}
                    ],
                };
                processor.process();
                data = processor.data;
            }
            stream.push_data(data);

            stream.update();
        }
        
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
                None => break
            };
            let f2: &Frequency = match freqs.peek() {
                Some(d) => *d,
                None => break
            };
            let y1: f32 = height - (f1.volume * height * 0.25);
            let y2: f32 = height - (f2.volume * height * 0.25);

            let x1: f32 = (x / frequencies.len() as f32) * width;
            let x2: f32 = ( (x + 1.0) / frequencies.len() as f32 ) * width;

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

    std::io::stdin().read_line(&mut input)
        .ok()
        .expect("Couldn't read line");
        
    input.trim().to_string()
}
