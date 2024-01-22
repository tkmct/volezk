//! Implements IKNP oblivious transfer extension
//! Refer: https://www.iacr.org/archive/crypto2003/27290145/27290145.pdf
use ark_std::rand::Rng;
use rand::prelude::{thread_rng, ThreadRng};
use sha3::{Digest, Keccak256};

use crate::{
    block::{Block, Block128},
    channel::AbstractChannel,
    ot::{OTReceiver, OTResult, OTSender},
};

/// Key size. Base of will be performed for K times to send M keys.
// TODO: Fix to 128 later
const K: usize = 2;

// TODO: make this B generics cleaner
fn ot_ext_send<
    Receiver: OTReceiver,
    T,
    B: Block + Clone + Default,
    const M: usize,
    C: AbstractChannel,
>(
    receiver: &mut Receiver,
    values: [[T; 2]; M],
    channel: C,
) -> OTResult<()> {
    // K OT for M bits messages where K is key length = 128.
    // Ext sender acts as an OT receiver
    // Sample K-bits
    let mut rng = thread_rng();
    // let s_choices = (0..K).map(|_| rng.gen::<bool>()).collect::<Vec<_>>();
    //
    // sender supposed to receive t for 0-th column, u for 1-st column for testing
    let s_choices = vec![false, true];

    println!("s: {:?}", s_choices);
    // println!("ot-ext-send M: {}", M);
    // println!("ot-ext-send M/8+1 = {}", M / 8 + 1);

    // Perform K OT to receive K M-bits column
    let mut q_matrix = vec![vec![false; K]; M];
    for (i, s) in s_choices.iter().enumerate() {
        let received = receiver.receive::<2, B, ThreadRng>(*s as usize, &mut rng)?;
        // println!("received {:?}", received);
        // convert received block back to M bit vec
        // let bytes = &received.as_bytes()[0..(M / 8 + 1)];
        let bytes = &received.as_bytes()[0..(M / 8 + 1)];
        println!("Received: {:?}", bytes);

        for j in 0..M {
            let l = j / 8;
            let byte = bytes[l];
            // println!("byte, {}", byte >> (7 - i % 8) & 1);

            println!("{}-th bit of q is {}", j, byte >> (7 - j % 8) & 1);
            q_matrix[j][i] = (byte >> (7 - j % 8) & 1) != 0;
        }
    }

    // compute M key pairs by hashing
    let keys = q_matrix
        .iter()
        .zip(s_choices)
        .map(|(row, s)| {
            let mut hasher_0 = Keccak256::default();
            let mut hasher_1 = Keccak256::default();
            let mut k_0_bytes = vec![0u8; K];
            let mut k_1_bytes = vec![0u8; K];

            for (i, &bit) in row.iter().enumerate() {
                let byte = i / 8;
                let shift = 7 - i % 8;
                k_0_bytes[byte] |= (bit as u8) << shift;
            }
            hasher_0.update(k_0_bytes);
            let k0 = hasher_0.finalize();

            for (i, &bit) in row.iter().enumerate() {
                let bit = bit ^ s;
                let byte = i / 8;
                let shift = 7 - i % 8;
                k_1_bytes[byte] |= (bit as u8) << shift;
            }
            hasher_1.update(k_1_bytes);
            let k1 = hasher_1.finalize();

            (k0, k1)
        })
        .collect::<Vec<_>>();

    println!("q_matrix: {:?}", q_matrix);
    // todo!()
    Ok(())
}

/// Const M is number of items to receive from sender
/// T is a type of value to receive from sender
/// B is a Block type to represent t, u, q matrix
fn ot_ext_receive<
    Sender: OTSender,
    T,
    B: Block + Clone,
    const M: usize,
    C: AbstractChannel,
    R: Rng,
>(
    sender: &mut Sender,
    choices: [bool; M],
    channel: C,
    rng: &mut R,
    // ) -> OTResult<[T; M]> {
) -> OTResult<()> {
    // K OT for M-bits messages where K is key length = 128.
    // Ext receiver acts as an OT sender
    // 1. create M * K matrix where all the values for i'th row are choice bit b_i.
    let b_matrix = choices.iter().map(|&b_i| vec![b_i; K]).collect::<Vec<_>>();
    println!("b_matrix: {:?}", b_matrix);

    // 2. Sample random M * K matrix t_matrix to form share of b_matrix.
    // t_matrix ^ u_matrix = b_matrix
    let mut rng = thread_rng();
    // let t_matrix = (0..M)
    //     .map(|_| {
    //         let t_i: u128 = rng.gen();
    //         let mut row = [false; K];
    //         row.iter_mut()
    //             .enumerate()
    //             .take(K)
    //             .map(|(i, _)| (t_i >> i) & 1 == 1)
    //             .collect::<Vec<_>>()
    //     })
    //     .collect::<Vec<_>>();

    // For testing purpose
    let t_matrix = vec![vec![true, false], vec![false, true]];

    println!("t_matrix: {:?}", t_matrix);

    let u_matrix = b_matrix
        .iter()
        .zip(&t_matrix)
        .map(|(b_row, t_row)| {
            b_row
                .iter()
                .zip(t_row)
                .map(|(b, t)| b ^ t)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    println!("u_matrix: {:?}", u_matrix);

    // perform K OT to send K-columns either (t_col, u_col)
    // send M-bits as Vec<Block128>
    // For now, This OT happens sequentially.
    // improve performance by making this OT paralelly
    for i in 0..K {
        let t_col = t_matrix.iter().map(|t_row| t_row[i]).collect::<Vec<_>>();
        let mut t_bytes = vec![0u8; M / 8 + 1];
        for (i, bit) in t_col.into_iter().enumerate() {
            let byte = i / 8;
            let shift = 7 - i % 8;
            t_bytes[byte] |= (bit as u8) << shift;
        }
        let t_block = B::from_bytes(&t_bytes);

        let u_col = u_matrix.iter().map(|u_row| u_row[i]).collect::<Vec<_>>();
        let mut u_bytes = vec![0u8; M / 8 + 1];
        for (i, bit) in u_col.into_iter().enumerate() {
            let byte = i / 8;
            let shift = 7 - i % 8;
            u_bytes[byte] |= (bit as u8) << shift;
        }
        let u_block = B::from_bytes(&u_bytes);

        // println!("t_block: {:?}", &t_block);
        // println!("u_block: {:?}", &u_block);

        // sender.send([t_block, u_block])?;

        sender.send([t_block, u_block])?;
        println!("ot-ext-recv Send T and U");
    }

    Ok(())
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
        let (ot_sender_stream, ot_receiver_stream) = UnixStream::pair().unwrap();
        let (ext_sender_stream, ext_receiver_stream) = UnixStream::pair().unwrap();

        let receiver_handle = thread::spawn(move || {
            let mut rng = thread_rng();
            let reader = BufReader::new(ot_sender_stream.try_clone().unwrap());
            let writer = BufWriter::new(ot_sender_stream);
            let sender_channel = Channel::new(reader, writer);
            let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng).unwrap();
            // TODO: fix later
            let choices: [bool; 2] = [
                true, false,
                // true, true, false, true, true, false, true, false,
            ];

            let reader = BufReader::new(ext_receiver_stream.try_clone().unwrap());
            let writer = BufWriter::new(ext_receiver_stream);
            let ext_receiver_chan = Channel::new(reader, writer);

            ot_ext_receive::<
                CO15Sender<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
                Block128,
                // TODO: fix later
                [Block128; 1],
                2,
                Channel<BufReader<UnixStream>, BufWriter<UnixStream>>,
                ThreadRng,
            >(&mut ot_sender, choices, ext_receiver_chan, &mut rng)
        });

        // Prepare sender
        let reader = BufReader::new(ot_receiver_stream.try_clone().unwrap());
        let writer = BufWriter::new(ot_receiver_stream);
        let receiver_channel = Channel::new(reader, writer);
        let mut ot_receiver = CO15Receiver::setup(receiver_channel).unwrap();

        // TODO: fix later
        let values: [[Block128; 2]; 2] = [
            [Block128::from(1), Block128::from(100)],
            [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
            // [Block128::from(1), Block128::from(100)],
        ];
        let reader = BufReader::new(ext_sender_stream.try_clone().unwrap());
        let writer = BufWriter::new(ext_sender_stream);
        let mut ext_sender_chan = Channel::new(reader, writer);

        ot_ext_send::<
            CO15Receiver<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
            Block128,
            // TODO: fix later
            [Block128; 1],
            2,
            Channel<BufReader<UnixStream>, BufWriter<UnixStream>>,
        >(&mut ot_receiver, values, ext_sender_chan)?;

        let receiver_result = receiver_handle.join().unwrap();
        assert!(receiver_result.is_ok());

        // TODO: fix later
        let expected_result = [
            Block128::from(1),
            Block128::from(100),
            // Block128::from(1),
            // Block128::from(1),
            // Block128::from(100),
            // Block128::from(1),
            // Block128::from(1),
            // Block128::from(100),
            // Block128::from(1),
            // Block128::from(100),
        ];

        // assert_eq!(receiver_result.unwrap(), expected_result);

        Ok(())
    }
}
