//! This module implement VOLE from oblivious transfer.

use crate::AbstractChannel;

struct OleLeader<C: AbstractChannel> {
    channel: C,
}

impl<C: AbstractChannel> OleLeader<C> {
    fn new(channel: C) -> Self {
        Self { channel }
    }

    fn execute(&mut self) {
        todo!()
    }
}

struct OleFollower<C: AbstractChannel> {
    channel: C,
}

impl<C: AbstractChannel> OleFollower<C> {
    fn new(channel: C) -> Self {
        Self { channel }
    }

    fn execute(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
        thread,
    };

    use super::*;
    use crate::channel::Channel;

    #[test]
    fn test_ole() {
        let (leader_stream, follower_stream) = UnixStream::pair().unwrap();

        let leader_thread = thread::spawn(|| {
            let reader = BufReader::new(leader_stream.try_clone().unwrap());
            let writer = BufWriter::new(leader_stream);
            let leader_chan = Channel::new(reader, writer);

            let mut alice = OleLeader::new(leader_chan);
            alice.execute()
        });

        let reader = BufReader::new(follower_stream.try_clone().unwrap());
        let writer = BufWriter::new(follower_stream);
        let follower_chan = Channel::new(reader, writer);

        let mut bob = OleFollower::new(follower_chan);
        bob.execute();

        // follower things to do
        let leader_result = leader_thread.join().unwrap();
    }
}
