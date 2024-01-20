//! Implements IKNP oblivious transfer extension
//! Refer: https://www.iacr.org/archive/crypto2003/27290145/27290145.pdf
use ark_std::rand::Rng;
use rand::prelude::thread_rng;

use crate::{
    block::Block,
    ot::{OTResult, ROTReceiver, ROTSender},
};

/// Key size. Base of will be performed for K times to send M keys.
const K: usize = 128;

fn ot_ext_send<Receiver: ROTReceiver, T, const M: usize>(
    receiver: &mut Receiver,
    values: [[T; 2]; M],
) -> OTResult<()> {
    todo!()
    // K OT for M bits messages where K is key length = 128.
    // Ext sender acts as an OT receiver
}

fn ot_ext_receive<Sender: ROTSender, T, const M: usize, R: Rng>(
    sender: &mut Sender,
    choices: [bool; M],
    rng: &mut R,
) -> OTResult<[T; M]> {
    // K OT for M bits messages where K is key length = 128.
    // Ext receiver acts as an OT sender
    // 1. create M * K matrix where all the values for i'th row are choice bit b_i.
    let b_matrix = choices.iter().map(|&b_i| vec![b_i; K]).collect::<Vec<_>>();

    // 2. Sample random M * K matrix t_matrix to form share of b_matrix.
    // t_matrix ^ u_matrix = b_matrix
    let mut rng = thread_rng();
    let t_matrix = (0..M).map(|_| {
        let t_i: u128 = rng.gen();
        let mut row = [false; 128];
        row.iter_mut()
            .enumerate()
            .map(|(i, _)| (t_i >> i) & 1 == 1)
            .collect::<Vec<_>>()
    });

    println!("t_matrix: {:?}", t_matrix);

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
            let sender_channel = Channel::new(reader, writer);
            let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng).unwrap();
            let choices: [bool; 10] = [
                true, false, true, true, false, true, true, false, true, false,
            ];

            ot_ext_receive::<
                CO15Sender<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
                Block128,
                10,
                ThreadRng,
            >(&mut ot_sender, choices, &mut rng)
        });

        // Prepare sender
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let receiver_channel = Channel::new(reader, writer);
        let mut ot_receiver = CO15Receiver::setup(receiver_channel).unwrap();
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
            CO15Receiver<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
            Block128,
            10,
        >(&mut ot_receiver, values)?;

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
