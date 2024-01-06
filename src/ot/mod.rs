//! Implement 1-of-n oblivious transfer trait
use ark_serialize::SerializationError;

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
    fn send<T>(&self, values: [T; N]) -> OTResult<T>;
}

pub trait OTReceiver {
    fn receive<T>(&self, choice: usize) -> OTResult<T>;
}
