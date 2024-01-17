//! Implement 1-of-n oblivious transfer trait
use ark_serialize::SerializationError;
use ark_std::rand::Rng;

use crate::{block::Block, channel::ChannelError};

mod co15;
mod extension;

#[derive(thiserror::Error, Debug)]
pub enum OTError {
    #[error(transparent)]
    Serialize {
        #[from]
        source: SerializationError,
    },
    #[error(transparent)]
    Channel {
        #[from]
        source: ChannelError,
    },
}

type OTResult<T> = Result<T, OTError>;

pub trait OTSender {
    fn send<const N: usize, T>(&mut self, values: [T; N]) -> OTResult<()>
    where
        T: Block + Clone;
}

pub trait OTReceiver {
    fn receive<const N: usize, T, R>(&mut self, choice: usize, rng: &mut R) -> OTResult<T>
    where
        T: Block + Clone + Copy + Default,
        R: Rng;
}

/// Random OT sender
pub trait ROTSender {
    fn send<const N: usize, T>(&mut self) -> OTResult<[T; N]>;
}

/// Random OT receiver
pub trait ROTReceiver {
    fn receive<const N: usize, T, R>(&mut self, choice: usize) -> OTResult<T>;
}
