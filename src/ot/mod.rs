//! Implement 1-of-n oblivious transfer trait
mod co15;

use crate::channel::AbstractChannel;

#[derive(thiserror::Error, Debug)]
enum OTError {}

type OTResult<T> = Result<T, OTError>;

trait OTSender<T, const N: usize> {
    fn send<C: AbstractChannel>(&self, channel: &mut C, values: [T; N]) -> OTResult<T>;
}

trait OTReceiver<T> {
    fn receive<C: AbstractChannel>(&self, channel: &mut C, choice: usize) -> OTResult<T>;
}
