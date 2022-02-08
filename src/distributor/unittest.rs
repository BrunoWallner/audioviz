#![allow(dead_code)]

#[cfg(feature = "std")]
pub(crate) fn test() {
    use std::{time::Duration, thread::sleep};
    use super::Distributor;

    let estimated_sample_rate: f64 = 8.0 * 20.0 / 5.0;
    let mut distributor: Distributor<u128> = Distributor::new(estimated_sample_rate);

    let mut counter: u128 = 0;
    'distribution: loop {
        if counter % 5 == 0 {
            let mut buffer: Vec<u128> = Vec::new();
            for i in 0..=8 {
                buffer.push(counter + i);
            }

            distributor.push_auto(&buffer);
        }

        let data = distributor.pop_auto();

        // if sample rate is fully known with 2 pushes
        if counter >= 10 {
            assert!(data.len() > 0);
            let buf_len = distributor.clone_buffer().len();
            assert!(buf_len <= 16);
        }

        counter += 1;
        sleep(Duration::from_millis(1));

        if counter > 100 {
            break 'distribution;
        }
    }
}