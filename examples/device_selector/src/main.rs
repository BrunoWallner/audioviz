use audioviz::audio_capture::{
    capture::Capture,
    config::Config as CaptureConfig
};

fn main() {
    let devices = Capture::fetch_devices().unwrap();
    println!("ID       Device");
    println!("------------------");
    for (i, dev) in devices.iter().enumerate() {
        println!("{i}\t {}", dev);
    }

    let id = match input("id: ").parse::<usize>() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("invalid input");
            std::process::exit(1);
        }
    };
    let device = devices[id].clone();

    println!("capturing audio from: {device}");
    let config = CaptureConfig {
        device,
        ..Default::default()
    };
    
    // must be in scope, otherwise capture will stop
    let _capture = Capture::init(config).unwrap();
    loop {}
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
