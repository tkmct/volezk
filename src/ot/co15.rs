//! This module implements oblivious trasnfer implementation described in
//! https://eprint.iacr.org/2015/267.pdf by Tung Chou and Claudio Orlandi

use ark_ec::{twisted_edwards::TECurveConfig, CurveConfig, CurveGroup};
use ark_ed25519::{EdwardsConfig, EdwardsProjective as G};
use ark_ff::{Field, Fp, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{rand::Rng, UniformRand, Zero};

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

impl<C: AbstractChannel, const N: usize> OTSender<N> for CO15Sender<C> {
    fn send<T>(&self, values: [T; N]) -> OTResult<T> {
        // Samples y from Z_p
        // Compute S = yB, T = yS
        // where B is a generator of a prime group.
        // send S to receiver
        //
        // samples x from Z_p
        //
        // Compute R = cS + xB
        // where c is a choice
        //
        //
        // Group G is subset of points over twisted Edwards curve.
        // −x^2 + y^2 = 1 + d x^2 y^2
        // constant d and generator B can be found in https://eprint.iacr.org/2011/368.pdf
        // the implementation comes from ark-works/ed25519
        // d =

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
    fn receive<T>(&self, choice: usize) -> OTResult<T> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
    };

    use ark_ec::{twisted_edwards::TECurveConfig, CurveConfig, CurveGroup};
    use ark_ed25519::{EdwardsConfig, EdwardsProjective as G};
    use ark_ff::{Field, Fp, PrimeField};
    use ark_std::test_rng;
    use ark_std::{UniformRand, Zero};

    use super::*;
    use crate::channel::Channel;

    #[test]
    #[ignore]
    fn test_ot() -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = test_rng();
        let (sender, receiver) = UnixStream::pair().unwrap();

        // Prepare sender
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let sender_channel = Channel::new(reader, writer);
        let ot_sender = CO15Sender::setup(sender_channel, &mut rng)?;

        // Preapre receiver
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let receiver_channel = Channel::new(reader, writer);
        let ot_receiver = CO15Receiver::setup(receiver_channel)?;

        let values: [u32; 2] = [1, 100];
        let choice = 1;

        ot_sender.send(values)?;
        let receive_result = ot_receiver.receive::<u32>(choice)?;

        assert_eq!(receive_result, 100);

        Ok(())
    }

    #[test]
    fn test_add() {
        // let mut rng = test_rng();

        let b = EdwardsConfig::GENERATOR;
        // let scalar = <EdwardsConfig as CurveConfig>::ScalarField::rand(&mut rng);
        let two = <EdwardsConfig as CurveConfig>::ScalarField::from(2);

        // let s = b * two;

        assert_eq!(b * two, b + b);
    }
}
