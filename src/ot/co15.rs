//! This module implements oblivious trasnfer implementation described in
//! https://eprint.iacr.org/2015/267.pdf by Tung Chou and Claudio Orlandi
use ark_ec::{twisted_edwards::TECurveConfig, CurveConfig, CurveGroup};
use ark_ed25519::{EdwardsConfig, EdwardsProjective as G};
use ark_ff::{Field, Fp, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{rand::Rng, UniformRand};

use crate::channel::AbstractChannel;

use super::{OTReceiver, OTResult, OTSender};

type Zp = <EdwardsConfig as CurveConfig>::ScalarField;

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
        let mut buff = Vec::new();
        s.serialize_compressed(&mut buff)?;
        channel.write_bytes(&buff)?;

        Ok(Self { channel, y, s, t })
    }
}

impl<C: AbstractChannel> OTSender for CO15Sender<C> {
    fn send<const N: usize, T, R: Rng>(&mut self, values: [T; N], rng: &mut R) -> OTResult<T> {
        // Receive rs from receiver
        let mut buff = Vec::new();
        self.channel.read_bytes(&mut buff)?;

        // generate keys using rs

        todo!()
    }
}

pub struct CO15Receiver<C: AbstractChannel> {
    channel: C,
    s: G,
}

impl<C: AbstractChannel> CO15Receiver<C> {
    fn setup(mut channel: C) -> OTResult<Self> {
        // setup something
        // receive s value from sender
        let mut buff = Vec::new();
        channel.read_bytes(&mut buff)?;

        // check if s is an element of G
        let s = G::deserialize_uncompressed(&*buff)?;
        Ok(Self { channel, s })
    }
}

impl<C: AbstractChannel> OTReceiver for CO15Receiver<C> {
    fn receive<const N: usize, T, R: Rng>(&mut self, choice: usize, rng: &mut R) -> OTResult<T> {
        // sample x from Z_p for N times
        // Compute R = cS + xB
        // where c is a choice
        let xs: [Zp; N] = std::array::from_fn(|_| Zp::rand(rng));
        let b = EdwardsConfig::GENERATOR;

        let rs = xs
            .iter()
            .enumerate()
            .map(|(i, x)| self.s * Zp::from(i as u32) + b * x)
            .collect::<Vec<_>>();

        // send rs to Receiver
        let mut buff = Vec::new();
        rs.serialize_compressed(&mut buff)?;
        self.channel.write_bytes(&buff)?;

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
    };

    use rand::prelude::{thread_rng, ThreadRng};

    use ark_ec::{twisted_edwards::TECurveConfig, CurveConfig};
    use ark_ed25519::{EdwardsConfig, EdwardsProjective as G};
    use ark_std::test_rng;

    use super::*;
    use crate::channel::Channel;

    #[test]
    #[ignore]
    fn test_ot() -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = thread_rng();
        let (sender, receiver) = UnixStream::pair().unwrap();

        // Prepare sender
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let sender_channel = Channel::new(reader, writer);
        let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng)?;

        // Preapre receiver
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let receiver_channel = Channel::new(reader, writer);
        let mut ot_receiver =
            CO15Receiver::<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>::setup(
                receiver_channel,
            )?;

        let values: [u32; 2] = [1, 100];
        let choice = 1;

        ot_sender.send(values, &mut rng)?;
        let receive_result: OTResult<u32> =
            ot_receiver.receive::<2, u32, ThreadRng>(choice, &mut rng);

        assert_eq!(receive_result.unwrap(), 100);

        Ok(())
    }

    #[test]
    fn test_1() -> Result<(), Box<dyn std::error::Error>> {
        // let mut rng = test_rng();

        let b = EdwardsConfig::GENERATOR;
        // let scalar = <EdwardsConfig as CurveConfig>::ScalarField::rand(&mut rng);
        let two = <EdwardsConfig as CurveConfig>::ScalarField::from(2);

        let s = b * two;

        assert_eq!(b * two, b + b);

        let mut buff = Vec::new();
        s.serialize_compressed(&mut buff)?;

        let c = G::deserialize_compressed(&*buff)?;

        assert_eq!(s, c);

        Ok(())
    }
}
