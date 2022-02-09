//! Can receive buffers of any length, calculates it data rate
//! and then determines how much data it should give away on each `pop()` call
//! 
//! different delays and buffer lengths between buffer requests are no problem
//! 
//! data rate should be known beforehand, but if this is not the case it is
//! recommended to manually call `clear()` after the second time data got pushed into it, 
//! before it is done forcefully

//! # Example with manual time measurement
//! ```
//! use audioviz::distributor::{Distributor, Elapsed};
//! use std::{time::{Duration, Instant}, thread::sleep};
//! 
//! fn main() {
//!     // neccessarry for distribution before data got pushed a second time
//!     // because sample rate is impossible to calculate with only one push
//!     // wrong rate results in bigger latency, which can be circumvented by
//!     // calling `distributor.clear()` after second push
//!     // * 8 because we push 8 items each round,
//!     // * 1000 because it loops 1000 times per second
//!     // / 5 because we only push every 5th loop
//!     let estimated_data_rate: f64 = 8.0 * 1000.0 / 5.0;
//!     let mut distributor: Distributor<u128> = Distributor::new(estimated_data_rate);
//! 
//!     let mut delta_push: Instant = Instant::now();
//!     let mut delta_pop: Instant = Instant::now();
//! 
//!     let mut counter: u128 = 0;
//!     'distribution: loop {
//!         if counter % 5 == 0 {
//!             let mut buffer: Vec<u128> = Vec::new();
//!             for i in 0..=8 {
//!                 buffer.push(counter + i);
//!             }
//! 
//!             let elapsed = delta_push.elapsed().as_micros();
//!             delta_push = Instant::now();
//!             distributor.push(&buffer, Elapsed::Micros(elapsed));
//!         }
//! 
//!         let data_rate = distributor.data_rate;
//!         let whole_data = distributor.clone_buffer();
//! 
//!         let elapsed = delta_pop.elapsed().as_micros();
//!         let data = distributor.pop(Elapsed::Micros(elapsed));
//!         delta_pop = Instant::now();
//! 
//!         println!("data_rate     : {}", data_rate);
//!         println!("whole data      : {:?}", whole_data);
//!         println!("distributed data: {:?}\n", data);
//! 
//!         counter += 1;
//!         sleep(Duration::from_millis(1));
//! 
//!         if counter > 50 {
//!             break 'distribution;
//!         }
//!     }
//! }
//! 
//! ```
//! 
//! If the `std` feature is enabled, time measurement can done by the Distributor itself
//! using `pop_auto()` and `push_auto()`
#[cfg(feature = "std")]
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Distributor<T> {
    last_buffer_size: usize,
    last_pop_size: usize,

    /// in data bits per second, (Hz)
    pub data_rate: f64,

    fully_initialized: bool,

    // neccessarry for even better distribution
    send_amount_excess: f64,
    pub buffer: Vec<T>,

    #[cfg(feature = "std")]
    push_elapsed: Instant,

    #[cfg(feature = "std")]
    pop_elapsed: Instant,
}

pub enum Elapsed {
    Nanos(u128),
    Micros(u128),
    Millis(u64)
}

impl<T: Clone> Distributor<T> {
    pub fn new(estimated_data_rate: f64) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            last_buffer_size: 0,
            last_pop_size: 0,
            data_rate: estimated_data_rate,

            fully_initialized: false,
            send_amount_excess: 0.0,
            buffer: Vec::new(),
        };

        #[cfg(feature = "std")]
        return Self {
            last_buffer_size: 0,
            last_pop_size: 0,
            data_rate: estimated_data_rate,

            fully_initialized: false,
            send_amount_excess: 0.0,
            buffer: Vec::new(),

            push_elapsed: Instant::now(),
            pop_elapsed: Instant::now(),
        };
    }

    pub fn clone_buffer(&self) -> Vec<T> {
        self.buffer.clone()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn push(&mut self, buffer: &[T], elapsed: Elapsed) {
        self.last_buffer_size = buffer.len();

        if self.fully_initialized {
            self.data_rate = match elapsed {
                Elapsed::Nanos(elapsed) => (buffer.len() - self.last_pop_size) as f64 / elapsed as f64 * 1_000_000_000.0,
                Elapsed::Micros(elapsed) => (buffer.len() - self.last_pop_size) as f64 / elapsed as f64 * 1_000_000.0,
                Elapsed::Millis(elapsed) => (buffer.len() - self.last_pop_size) as f64 / elapsed as f64 * 1_000.0,
            } 
        }

        self.buffer.append(&mut buffer.to_vec());
        self.fully_initialized = true;
    }

    #[cfg(feature = "std")]
    /// same as `push()` but with automatic time measurement
    pub fn push_auto(&mut self, buffer: &[T]) {
        self.last_buffer_size = buffer.len();

        let elapsed = self.push_elapsed.elapsed().as_micros();
        self.push_elapsed = Instant::now();

        if self.fully_initialized {
            self.data_rate = (buffer.len() - self.last_pop_size) as f64 / elapsed as f64 * 1_000_000.0;
        }

        self.buffer.append(&mut buffer.to_vec());
        self.fully_initialized = true;
    }
    /// array length is unknown and dependent on sample rate and the interval between `pop()` calls
    pub fn pop(&mut self, elapsed: Elapsed) -> Vec<T> {
        // calculates what amount to send for continous stream
        //let send_amount: usize = ( (elapsed as f64 / 1_000_000.0 /* to convert from µs to s */) * self.data_rate ).round() as usize;
        let send_amount: f64 = match elapsed {
            Elapsed::Nanos(elapsed) => (elapsed as f64 / 1_000_000_000.0 /* to convert from ns to s */) * self.data_rate,
            Elapsed::Micros(elapsed) => (elapsed as f64 / 1_000_000.0) * self.data_rate,
            Elapsed::Millis(elapsed) => (elapsed as f64 / 1_000.0) * self.data_rate,
        };
        self.send_amount_excess += send_amount % 1.0;
        let mut send_amount = send_amount.floor() as usize;

        // handle of send_amount_excess
        if self.send_amount_excess >= 1.0 {
            send_amount += 1;
            self.send_amount_excess -= 1.0;
        }

        let o_buffer: Vec<T>;
        if self.buffer.len() > send_amount {
            o_buffer = self.buffer[0..send_amount].to_vec();
            self.buffer.drain(0..send_amount);
        } else {
            o_buffer = self.buffer.clone();
            self.buffer.drain(..);
        }

        // prevents buffer to grow indefinetly, can happeen when
        // distributor runs for hours
        let cap: usize = self.last_buffer_size * 2;
        if self.buffer.len() > cap && cap != 0 {
            log::warn!("force reset of distribution buffer");
            if self.buffer.len() > send_amount {
                let oversize: usize = self.buffer.len() - send_amount;
                self.buffer.drain(0..oversize);
            }
        }

        o_buffer
    }

    #[cfg(feature = "std")]
    /// same as `pop()` but with automatic time measurement
    pub fn pop_auto(&mut self) -> Vec<T> {
        // calculates what amount to send for continous stream
        let elapsed = self.pop_elapsed.elapsed().as_micros();
        self.pop_elapsed = Instant::now();

        let send_amount: f64 = (elapsed as f64 / 1_000_000.0) * self.data_rate;
        self.send_amount_excess += send_amount % 1.0;
        let mut send_amount = send_amount.floor() as usize;

        // handle of send_amount_excess
        if self.send_amount_excess >= 1.0 {
            send_amount += 1;
            self.send_amount_excess -= 1.0;
        }

        let o_buffer: Vec<T>;
        if self.buffer.len() > send_amount {
            o_buffer = self.buffer[0..send_amount].to_vec();
            self.buffer.drain(0..send_amount);
        } else {
            o_buffer = self.buffer.clone();
            self.buffer.drain(..);
        }

        // prevents buffer to grow indefinetly, can happeen when
        // distributor runs for hours
        let cap: usize = self.last_buffer_size * 2;
        if self.buffer.len() > cap && cap != 0 {
            log::warn!("force reset of distribution buffer");
            if self.buffer.len() > send_amount {
                let oversize: usize = self.buffer.len() - send_amount;
                self.buffer.drain(0..oversize);
            }
        }

        o_buffer
    }
}