//! Implement 1-of-n oblivious transfer trait
use ark_serialize::SerializationError;
use ark_std::rand::Rng;

use crate::{block::Block, channel::ChannelError};

pub mod co15;
pub mod extension;

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
        T: Block + Clone + Default,
        R: Rng;
}

/// Random OT sender
pub trait ROTSender {
    fn send_random<const N: usize, T: Block>(&mut self) -> OTResult<[T; N]>;
}

/// Random OT receiver
pub trait ROTReceiver {
    fn receive_random<const N: usize, T: Block, R: Rng>(
        &mut self,
        choice: usize,
        rng: &mut R,
    ) -> OTResult<T>;
}
