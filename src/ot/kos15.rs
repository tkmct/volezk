//! This module implements oblivious trasnfer implementation described in
//! https://eprint.iacr.org/2015/546.pdf

use crate::AbstractChannel;

pub struct Kos15Sender<C: AbstractChannel> {
    channel: C,
}

pub struct Kos15Receiver<C: AbstractChannel> {
    channel: C,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ot() {}
}
