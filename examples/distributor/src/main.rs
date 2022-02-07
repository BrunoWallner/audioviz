use audioviz::distributor::{Distributor, Elapsed};
use std::{thread::sleep, time::{Duration, Instant}};

fn main() {
    // neccessarry for distribution before data got pushed a second time
    // because it is impossible to calculate with only one push
    // * 8 because we push 8 items each round,
    // * 4 it loops 4 times per second
    // / 5 because we only push every 5th loop
    let estimated_sample_rate: f64 = 8.0 * 4.0 / 5.0;
    let mut distributor: Distributor<u128> = Distributor::new(estimated_sample_rate);

    let mut delta_push: Instant = Instant::now();
    let mut delta_pop: Instant = Instant::now();

    let mut counter: u128 = 0;
    loop {
        if counter % 5 == 0 {
            let mut buffer: Vec<u128> = Vec::new();
            for i in 0..=8 {
                buffer.push(counter + i);
            }

            let elapsed = delta_push.elapsed().as_micros();
            delta_push = Instant::now();
            distributor.push(&buffer, Elapsed::Micros(elapsed));
        }

        let sample_rate = distributor.sample_rate;
        let whole_data = distributor.clone_buffer();

        let elapsed = delta_pop.elapsed().as_micros();
        let data = distributor.pop(Elapsed::Micros(elapsed));
        delta_pop = Instant::now();

        println!("sample_rate     : {}", sample_rate);
        println!("whole data      : {:?}", whole_data);
        println!("distributed data: {:?}\n", data);

        counter += 1;
        sleep(Duration::from_millis(250));
    }
}
