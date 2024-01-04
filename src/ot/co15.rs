use crate::channel::AbstractChannel;

use super::{OTReceiver, OTResult, OTSender};

struct CO15Sender {}

impl<T, const N: usize> OTSender<T, N> for CO15Sender {
    fn send<C: AbstractChannel>(&self, channel: &mut C, values: [T; N]) -> OTResult<T> {
        todo!()
    }
}

struct CO15Receiver {}

impl<T> OTReceiver<T> for CO15Receiver {
    fn receive<C: AbstractChannel>(&self, channel: &mut C, choice: usize) -> OTResult<T> {
        todo!()
    }
}
