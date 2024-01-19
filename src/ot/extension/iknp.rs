//! Implements IKNP oblivious transfer extension
//! Refer: https://www.iacr.org/archive/crypto2003/27290145/27290145.pdf
use ark_std::rand::Rng;

use crate::{
    block::Block,
    ot::{OTResult, ROTReceiver, ROTSender},
};

fn ot_ext_send<Sender: ROTSender, T, const M: usize>(
    sender: &mut Sender,
    values: [[T; 2]; M],
) -> OTResult<()> {
    todo!()
}

fn ot_ext_receive<Receiver: ROTReceiver, T, const M: usize, R: Rng>(
    receiver: &mut Receiver,
    choices: [bool; M],
    rng: &mut R,
) -> OTResult<[T; M]> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
        thread,
    };

    use rand::{prelude::thread_rng, rngs::ThreadRng};

    use super::*;
    use crate::{
        block::*,
        channel::Channel,
        ot::{
            co15::{CO15Receiver, CO15Sender},
            OTError,
        },
    };

    use ark_ec::twisted_edwards::TECurveConfig;
    use ark_ed25519::EdwardsConfig;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use ark_std::UniformRand;

    #[test]
    fn test_ot_extension() -> Result<(), Box<dyn std::error::Error>> {
        // Do 128 base OT for key exchange
        let (sender, receiver) = UnixStream::pair().unwrap();

        let receiver_handle = thread::spawn(move || {
            let mut rng = thread_rng();
            let reader = BufReader::new(receiver.try_clone().unwrap());
            let writer = BufWriter::new(receiver);
            let receiver_channel = Channel::new(reader, writer);
            let mut ot_receiver = CO15Receiver::setup(receiver_channel).unwrap();
            let choices: [bool; 10] = [
                true, false, true, true, false, true, true, false, true, false,
            ];

            ot_ext_receive::<
                CO15Receiver<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
                Block128,
                10,
                ThreadRng,
            >(&mut ot_receiver, choices, &mut rng)
        });

        // Prepare sender
        let mut rng = thread_rng();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let sender_channel = Channel::new(reader, writer);
        let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng).unwrap();
        let values: [[Block128; 2]; 10] = [
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
        ];
        ot_ext_send::<
            CO15Sender<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
            Block128,
            10,
        >(&mut ot_sender, values)?;

        let receiver_result = receiver_handle.join().unwrap();
        assert!(receiver_result.is_ok());

        let expected_result = [
            Block128::from(1),
            Block128::from(100),
            Block128::from(1),
            Block128::from(1),
            Block128::from(100),
            Block128::from(1),
            Block128::from(1),
            Block128::from(100),
            Block128::from(1),
            Block128::from(100),
        ];

        assert_eq!(receiver_result.unwrap(), expected_result);

        Ok(())
    }
}
