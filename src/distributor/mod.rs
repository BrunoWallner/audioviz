//! Distributes buffered data, into multiple smaller buffers
//! 
//! It automatically detects sample rate and different delays between data requests are no problem

use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Distributor<T> {
    last_buffer_size: usize,
    sample_rate: f64, // in Hz
    last_send: Option<Instant>,
    last_request: Option<Instant>,

    pub data: Vec<T>,
}

impl<T: Clone> Distributor<T> {
    pub fn new() -> Self {
        Self {
            last_buffer_size: 0,
            sample_rate: 0.0,
            last_send: None,
            last_request: None,

            data: Vec::new(),
        }
    }

    pub fn push(&mut self, data: &[T]) {
        self.last_buffer_size = data.len();

        // calculates sample rate
        let elapsed: u128 = match self.last_send {
            Some(l) => l.elapsed().as_micros(),
            None => 0
        };
        self.last_send = Some(Instant::now());

        self.sample_rate = data.len() as f64 / elapsed as f64 * 1_000_000.0 /* to convert from µHz to Hz */;

        self.data.append(&mut data.to_vec());
    }

    /// array length is unknown and dependent on sample rate and the interval between `pop()` calls
    pub fn pop(&mut self) -> Vec<T> {
        let elapsed: u128 = match self.last_request {
            Some(l) => l.elapsed().as_micros(),
            None => 0
        };
        self.last_request = Some(Instant::now());

        // calculates what amount to send for continous stream
        let send_amount: usize = ( (elapsed as f64 / 1_000_000.0 /* to convert from µs to s */) * self.sample_rate ).ceil() as usize;

        let o_data: Vec<T>;
        if self.data.len() > send_amount {
            o_data = self.data[0..send_amount].to_vec();
            self.data.drain(0..send_amount);
        } else {
            o_data = self.data.clone();
        }

        // prevents buffer to grow indefinetly, can happeen when
        // distributor runs for hours
        let cap: usize = self.last_buffer_size * 2;
        if self.data.len() > cap && cap != 0 {
            log::warn!("force reset of distribution buffer");
            if self.data.len() > send_amount {
                let oversize: usize = self.data.len() - send_amount;
                self.data.drain(0..oversize);
            }
        }

        o_data
    }
}
