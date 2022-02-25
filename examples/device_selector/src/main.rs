use audioviz::audio_capture::capture::{Capture, Device};

fn main() {
    let mut audio_capture = Capture::new();
    let devices = audio_capture.fetch_devices().unwrap();
    println!("ID       Device");
    println!("------------------");
    for (i, dev) in devices.iter().enumerate() {
        println!("{}\t {}", i, dev);
    }

    let id = match input("id: ").parse::<usize>() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("invalid input");
            std::process::exit(1);
        }
    };
    let device = devices[id].clone();
    println!("capturing audio from: {}", device);
    
    // must be in scope, otherwise capture will stop
    let _capture = audio_capture.init(&Device::Id(id)).unwrap();
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
