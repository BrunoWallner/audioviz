use macroquad::prelude::*;

use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig, Interpolation}, stream::Stream};
use audioviz::distributor::Distributor;

#[macroquad::main("AudioSpectrum")]
async fn main() {
    let audio_capture = Capture::init(CaptureConfig::default()).unwrap();
    let audio_receiver = audio_capture.get_receiver().unwrap();

    let mut distributor: Distributor<f32> = Distributor::new(44_100.0);
    let stream_config: StreamConfig = StreamConfig {
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
        let data = distributor.pop_auto();
        stream.push_data(data);

        stream.update();
        
        let frequencies = stream.get_frequencies();

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
