pub mod input;
pub mod output;

pub use input::input::{Input, InputController};
pub use output::output::{Output, OutputController};

#[derive(Clone, Debug)]
pub enum Error {
    DeviceNotFound,
    DeviceNotAvailable,
    UnsupportedConfig,
    BackendSpecific(String),
}

#[derive(Clone, Debug)]
pub enum Device {
    DefaultInput,
    DefaultOutput,
    Id(usize),
}