//! Implement 1-of-n oblivious transfer trait
use ark_serialize::SerializationError;
use ark_std::rand::Rng;

use crate::{channel::ChannelError, types::*};

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

pub trait OTSender {
    fn send<const N: usize, T: Block + Clone>(&mut self, values: [T; N]) -> OTResult<()>;
}

pub trait OTReceiver {
    fn receive<const N: usize, T: Block + Clone, R: Rng>(
        &mut self,
        choice: usize,
        rng: &mut R,
    ) -> OTResult<T>;
}
