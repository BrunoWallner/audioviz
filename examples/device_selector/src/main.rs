use audioviz::io::{Input, Device};

fn main() {
    let mut audio_input = Input::new();
    let devices = audio_input.fetch_devices().unwrap();
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
    let _ = audio_input.init(&Device::Id(id)).unwrap();
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
