//! Implement 1-of-n oblivious transfer trait
use ark_serialize::SerializationError;
use ark_std::rand::Rng;

use crate::channel::ChannelError;

mod co15;

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

pub trait OTSender<const N: usize> {
    fn send<T, R: Rng>(&self, values: [T; N], rng: &mut R) -> OTResult<T>;
}

pub trait OTReceiver {
    fn receive<T>(&self, choice: usize) -> OTResult<T>;
}
