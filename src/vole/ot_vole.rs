//! This module implement VOLE from oblivious transfer and learning parity with noise assumption.

use crate::{
    ot::{OTReceiver, OTSender},
    AbstractChannel,
};

/// Vole sender inputs delta and outputs k
/// where y = k + x * Δ
struct VoleSender<C: AbstractChannel, Sender: OTSender> {
    ot_sender: Sender,
    channel: C,
}

impl<C: AbstractChannel, Sender: OTSender> VoleSender<C, Sender> {
    fn new(ot_sender: Sender, channel: C) -> Self {
        Self { channel, ot_sender }
    }

    fn send(&mut self) {
        todo!()
    }
}

/// Vole receiver inputs x and outputs y
/// where y = k + x * Δ
struct VoleReceiver<C: AbstractChannel, Receiver: OTReceiver> {
    ot_receiver: Receiver,
    channel: C,
}

impl<C: AbstractChannel, Receiver: OTReceiver> VoleReceiver<C, Receiver> {
    fn new(ot_receiver: Receiver, channel: C) -> Self {
        Self {
            channel,
            ot_receiver,
        }
    }

    fn receive(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::thread_rng;
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
        thread,
    };

    use super::*;
    use crate::{channel::Channel, ot::co15::*};

    #[test]
    fn test_ole() {
        let (sender_stream, receiver_stream) = UnixStream::pair().unwrap();
        let (ot_sender_stream, ot_receiver_stream) = UnixStream::pair().unwrap();

        let sender_thread = thread::spawn(|| {
            let mut rng = thread_rng();

            let reader = BufReader::new(sender_stream.try_clone().unwrap());
            let writer = BufWriter::new(sender_stream);
            let sender_chan = Channel::new(reader, writer);

            let ot_reader = BufReader::new(ot_sender_stream.try_clone().unwrap());
            let ot_writer = BufWriter::new(ot_sender_stream);
            let ot_sender_channel = Channel::new(ot_reader, ot_writer);
            let ot_sender = CO15Sender::setup(ot_sender_channel, &mut rng).unwrap();

            let mut alice = VoleSender::new(ot_sender, sender_chan);
            alice.send()
        });

        let reader = BufReader::new(receiver_stream.try_clone().unwrap());
        let writer = BufWriter::new(receiver_stream);
        let receiver_chan = Channel::new(reader, writer);

        let ot_reader = BufReader::new(ot_receiver_stream.try_clone().unwrap());
        let ot_writer = BufWriter::new(ot_receiver_stream);
        let ot_receiver_channel = Channel::new(ot_reader, ot_writer);
        let ot_receiver = CO15Receiver::setup(ot_receiver_channel).unwrap();

        let mut bob = VoleReceiver::new(ot_receiver, receiver_chan);
        bob.receive();

        // follower things to do
        let sender_result = sender_thread.join().unwrap();
    }
}
