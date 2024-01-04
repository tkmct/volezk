use crate::channel::AbstractChannel;

use super::{OTReceiver, OTResult, OTSender};

struct CO15Sender {}

impl CO15Sender {
    pub fn new() -> Self {
        CO15Sender {}
    }
}

impl<const N: usize> OTSender<N> for CO15Sender {
    fn send<C: AbstractChannel, T>(&self, channel: &mut C, values: [T; N]) -> OTResult<T> {
        todo!()
    }
}

struct CO15Receiver {}

impl CO15Receiver {
    pub fn new() -> Self {
        CO15Receiver {}
    }
}

impl OTReceiver for CO15Receiver {
    fn receive<C: AbstractChannel, T>(&self, channel: &mut C, choice: usize) -> OTResult<T> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
        str::Bytes,
    };

    use super::*;
    use crate::channel::Channel;

    #[test]
    fn test_ot() {
        let (sender, receiver) = UnixStream::pair().unwrap();

        // Prepare sender
        let ot_sender = CO15Sender::new();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let mut sender_channel = Channel::new(reader, writer);

        // Preapre receiver
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let ot_receiver = CO15Receiver::new();
        let mut receiver_channel = Channel::new(reader, writer);

        let values: [u32; 2] = [1, 100];
        let choice = 1;

        let send_result = ot_sender.send(&mut sender_channel, values);
        let receive_result = ot_receiver.receive::<_, u32>(&mut receiver_channel, choice);

        assert!(send_result.is_ok());
        assert!(receive_result.is_ok());
        assert_eq!(receive_result.unwrap(), 100);
    }
}
