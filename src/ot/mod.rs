//! Implement 1-of-n oblivious transfer trait
mod co15;

use crate::channel::AbstractChannel;

#[derive(thiserror::Error, Debug)]
pub enum OTError {}

type OTResult<T> = Result<T, OTError>;

pub trait OTSender<const N: usize> {
    fn send<C: AbstractChannel, T>(&self, channel: &mut C, values: [T; N]) -> OTResult<T>;
}

pub trait OTReceiver {
    fn receive<C: AbstractChannel, T>(&self, channel: &mut C, choice: usize) -> OTResult<T>;
}
