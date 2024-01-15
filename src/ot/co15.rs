//! This module implements oblivious trasnfer implementation described in
//! https://eprint.iacr.org/2015/267.pdf by Tung Chou and Claudio Orlandi
use ark_ec::twisted_edwards::TECurveConfig;
use ark_ed25519::EdwardsConfig;
use ark_serialize::CanonicalSerialize;
use ark_std::{rand::Rng, UniformRand};
use sha3::{Digest, Keccak256};

use crate::{block::*, channel::AbstractChannel, types::*};

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

        Ok(Self { channel, y, s, t })
    }
}

impl<C: AbstractChannel> OTSender for CO15Sender<C> {
    fn send<const N: usize, T: Block + Clone>(&mut self, values: [T; N]) -> OTResult<()> {
        // Receive r from receiver
        let r = self.channel.read_g()?;

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
        for (i, v) in values.iter().enumerate() {
            // for i in 0..N {
            let mut hasher = hasher.clone();
            let k = r * self.y - self.t * Zp::from(i as u32);
            let mut buff = Vec::new();
            k.serialize_compressed(&mut buff)?;

            hasher.update(buff);
            let key = hasher.finalize();
            let encrypted = v.encrypt(&key.into());

            // send ciphertext to receiver
            self.channel.write_bytes(&encrypted.as_bytes()).unwrap();
            self.channel.flush()?;
        }

        Ok(())
    }
}

pub struct CO15Receiver<C: AbstractChannel> {
    channel: C,
    s: G,
}

impl<C: AbstractChannel> CO15Receiver<C> {
    /// receive s value from sender
    pub fn setup(mut channel: C) -> OTResult<Self> {
        let s = channel.read_g()?;
        channel.flush()?;
        Ok(Self { channel, s })
    }
}

impl<C: AbstractChannel> OTReceiver for CO15Receiver<C> {
    fn receive<const N: usize, T, R>(&mut self, choice: usize, rng: &mut R) -> OTResult<T>
    where
        T: Block + Clone + Copy + Default,
        R: Rng,
    {
        // sample x from Z_p for N times
        // Compute R = cS + xB
        // where c is a choice
        let x = Zp::rand(rng);
        let b = EdwardsConfig::GENERATOR;
        let r = self.s * Zp::from(choice as u32) + b * x;
        self.channel.write_g(r)?;
        self.channel.flush()?;

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

        let ciphertexts: [T; N] = std::array::from_fn(|_| {
            let mut bytes = vec![0u8; T::BYTES_LEN];
            // TODO: handle result correctly
            self.channel.read_bytes(&mut bytes).unwrap();
            T::from_bytes(&bytes)
        });

        // decipher the choice ciphertext
        let dest = ciphertexts[choice];
        let res = dest.decrypt(&key.into());
        // TODO: handle the result

        Ok(res)
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
    use crate::{channel::Channel, ot::OTError};

    use ark_ec::twisted_edwards::TECurveConfig;
    use ark_ed25519::EdwardsConfig;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use ark_std::UniformRand;

    #[test]
    fn test_ot_block128() -> Result<(), Box<dyn std::error::Error>> {
        let (sender, receiver) = UnixStream::pair().unwrap();

        // Preapre receiver
        let receiver_handle = thread::spawn(move || {
            let mut rng = thread_rng();
            let reader = BufReader::new(receiver.try_clone().unwrap());
            let writer = BufWriter::new(receiver);
            let receiver_channel = Channel::new(reader, writer);
            let mut ot_receiver = CO15Receiver::setup(receiver_channel).unwrap();

            let choice = 1;
            ot_receiver.receive::<2, Block128, ThreadRng>(choice, &mut rng)
        });

        // Prepare sender
        let mut rng = thread_rng();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let sender_channel = Channel::new(reader, writer);
        let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng).unwrap();
        let values: [Block128; 2] = [Block128::from(1), Block128::from(100)];
        ot_sender.send(values)?;

        let receiver_result = receiver_handle.join().unwrap();
        assert!(receiver_result.is_ok());
        assert_eq!(receiver_result.unwrap(), values[1]);

        Ok(())
    }

    #[test]
    fn test_ot_block256() -> Result<(), Box<dyn std::error::Error>> {
        // 1 out-of 5 OT with points on G

        let (sender, receiver) = UnixStream::pair().unwrap();

        // Preapre receiver
        let receiver_handle = thread::spawn(move || {
            let mut rng = thread_rng();
            let reader = BufReader::new(receiver.try_clone().unwrap());
            let writer = BufWriter::new(receiver);
            let receiver_channel = Channel::new(reader, writer);
            let mut ot_receiver = CO15Receiver::setup(receiver_channel).unwrap();

            let choice = 3;
            let res = ot_receiver
                .receive::<5, Block256, ThreadRng>(choice, &mut rng)
                .unwrap();
            let g = G::deserialize_compressed(res.as_bytes().as_slice());
            g
        });

        // Prepare sender
        let mut rng = thread_rng();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let sender_channel = Channel::new(reader, writer);
        let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng).unwrap();

        let mut rng = thread_rng();
        let b = EdwardsConfig::GENERATOR;

        let points: [G; 5] = std::array::from_fn(|_| {
            let y = Zp::rand(&mut rng);
            b * y
        });

        let values: [Block256; 5] = points
            .iter()
            .map(|g| {
                let mut bytes = Vec::new();
                g.serialize_compressed(&mut bytes)?;

                Ok(Block256::from_bytes(&bytes))
            })
            .collect::<Result<Vec<_>, OTError>>()?
            .try_into()
            .unwrap();

        ot_sender.send(values)?;

        let receiver_result = receiver_handle.join().unwrap();
        assert!(receiver_result.is_ok());
        assert_eq!(receiver_result.unwrap(), points[3]);

        Ok(())
    }
}
