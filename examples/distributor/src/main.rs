use audioviz::distributor::{Distributor, Elapsed};
use std::{thread::sleep, time::{Duration, Instant}};

use rand::Rng;

fn main() {
    // neccessarry for distribution before data got pushed a second time
    // because it is impossible to calculate with only one push
    // * 8 because we push 8 items each round,
    // * 4 it loops 4 times per second
    // / 5 because we only push every 5th loop
    let estimated_data_rate: f64 = 8.0 * 4.0 / 5.0;
    let mut distributor: Distributor<u128> = Distributor::new(estimated_data_rate, Some(64));

    let mut delta_push: Instant = Instant::now();
    let mut delta_pop: Instant = Instant::now();

    let mut value: u128 = 0;
    loop {
        let rng = rand::thread_rng().gen_range(0..=4);
        if rng == 0 {
            let mut buffer: Vec<u128> = Vec::new();
            let len = rand::thread_rng().gen_range(1..16);
            for _ in 0..=len {
                buffer.push(value);
		        value += 1;
            }

            let elapsed = delta_push.elapsed().as_micros();
            delta_push = Instant::now();
            distributor.push(&buffer, Elapsed::Micros(elapsed));
        }

        let data_rate = distributor.data_rate;
        let whole_data = distributor.clone_buffer();

        let elapsed = delta_pop.elapsed().as_micros();
        let data = distributor.pop(Elapsed::Micros(elapsed), None);
        delta_pop = Instant::now();

        println!("data_rate     : {}", data_rate);
        println!("whole data      : {:?}", whole_data);
        println!("distributed data: {:?}\n", data);

        sleep(Duration::from_millis(250));
    }
}
