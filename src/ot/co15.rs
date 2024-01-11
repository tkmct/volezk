//! This module implements oblivious trasnfer implementation described in
//! https://eprint.iacr.org/2015/267.pdf by Tung Chou and Claudio Orlandi
use aes::cipher::generic_array::sequence::GenericSequence;
use aes::cipher::{generic_array::GenericArray, BlockCipher, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes256;

use ark_ec::twisted_edwards::TECurveConfig;
use ark_ed25519::EdwardsConfig;
use ark_serialize::CanonicalSerialize;
use ark_std::{rand::Rng, UniformRand};
use sha3::{Digest, Keccak256};

use crate::{channel::AbstractChannel, types::*};

use super::{OTReceiver, OTResult, OTSender};

pub struct CO15Sender<C: AbstractChannel> {
    channel: C,

    y: Zp,
    s: G,
    t: G,
}

impl<C: AbstractChannel> CO15Sender<C> {
    pub fn setup<R: Rng>(mut channel: C, rng: &mut R) -> OTResult<Self> {
        // Group G is subset of points over twisted Edwards curve.
        // −x^2 + y^2 = 1 + d x^2 y^2
        // constant d and generator B can be found in https://eprint.iacr.org/2011/368.pdf
        // the implementation comes from ark-works/ed25519
        //
        // Samples y from Z_p
        let y = Zp::rand(rng);
        let b = EdwardsConfig::GENERATOR;

        // Compute S = yB, T = yS
        // where B is a generator of group of prime order
        let s = b * y;
        let t = s * y;

        // Send s to receiver
        channel.write_g(s)?;
        channel.flush()?;
        let g = channel.read_g()?;

        Ok(Self { channel, y, s, t })
    }
}

impl<C: AbstractChannel> OTSender for CO15Sender<C> {
    fn send<const N: usize, T, R: Rng>(&mut self, values: [T; N], rng: &mut R) -> OTResult<()> {
        // Receive r from receiver
        println!("receive r");
        let r = self.channel.read_g()?;
        println!("r: {:?}", r);

        let mut r_buff = Vec::new();
        let mut s_buff = Vec::new();

        r.serialize_compressed(&mut r_buff)?;
        self.s.serialize_compressed(&mut s_buff)?;

        let mut hasher = Keccak256::default();
        hasher.update(s_buff);
        hasher.update(r_buff);

        // Receive r from receiver
        // calculate keys using r
        // k_j = H (S,R )(yR − jT)
        let keys = (0..N)
            .map(|j| {
                let mut hasher = hasher.clone();
                let k = r * self.y - self.t * Zp::from(j as u32);
                let mut buff = Vec::new();
                k.serialize_compressed(&mut buff)?;

                hasher.update(buff);
                let key = hasher.finalize();

                let cipher = Aes256::new(&key);
                cipher.encrypt_block(values[j].into())
            })
            .collect::<OTResult<Vec<_>>>()?;

        println!("keys: {:?}", keys);

        todo!()
    }
}

pub struct CO15Receiver<C: AbstractChannel> {
    channel: C,
    s: G,
}

impl<C: AbstractChannel> CO15Receiver<C> {
    pub fn setup(mut channel: C) -> OTResult<Self> {
        // setup something
        // receive s value from sender
        let s = channel.read_g()?;
        channel.flush()?;
        channel.write_g(s)?;

        Ok(Self { channel, s })
    }
}

impl<C: AbstractChannel> OTReceiver for CO15Receiver<C> {
    fn receive<const N: usize, T, R: Rng>(&mut self, choice: usize, rng: &mut R) -> OTResult<T> {
        // sample x from Z_p for N times
        // Compute R = cS + xB
        // where c is a choice
        let x = Zp::rand(rng);
        let b = EdwardsConfig::GENERATOR;
        let r = self.s * Zp::from(choice as u32) + b * x;
        self.channel.write_g(r)?;

        let k = self.s * x;

        // calculate key
        let mut r_buff = Vec::new();
        let mut s_buff = Vec::new();
        let mut k_buff = Vec::new();

        r.serialize_compressed(&mut r_buff)?;
        self.s.serialize_compressed(&mut s_buff)?;
        k.serialize_compressed(&mut k_buff)?;

        let mut hasher = Keccak256::default();
        hasher.update(s_buff);
        hasher.update(r_buff);
        hasher.update(k_buff);

        let key = hasher.finalize();

        // encrypted values from sender
        // std::array::from_fn(|i| {
        //     self.channel.read_bytes();
        // });

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

    use rand::prelude::{thread_rng, ThreadRng};

    use super::*;
    use crate::channel::Channel;

    #[test]
    fn test_ot() -> Result<(), Box<dyn std::error::Error>> {
        let (sender, receiver) = UnixStream::pair().unwrap();

        // Preapre receiver
        let receiver_handle = thread::spawn(move || {
            let mut rng = thread_rng();
            let reader = BufReader::new(receiver.try_clone().unwrap());
            let writer = BufWriter::new(receiver);
            let receiver_channel = Channel::new(reader, writer);
            let mut ot_receiver = CO15Receiver::setup(receiver_channel).unwrap();

            let values: [u32; 2] = [1, 100];
            let choice = 1;

            // ot_sender.send(values, &mut rng)?;
            let receive_result = ot_receiver.receive::<2, u32, ThreadRng>(choice, &mut rng);
        });

        // let sender_handle = thread::spawn(move || {
        // Prepare sender
        let mut rng = thread_rng();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let sender_channel = Channel::new(reader, writer);
        let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng).unwrap();
        let values: [u32; 2] = [1, 100];
        ot_sender.send(values, &mut rng)?;
        // });

        let sender_result = receiver_handle.join();
        assert!(sender_result.is_ok());
        let s = sender_result.unwrap();

        // assert_eq!(s, ot_receiver.s);

        // println!("sender {:?}, receiver {:?}", s, ot_receiver.s);

        Ok(())
    }
}
