use ark_ec::CurveConfig;
use ark_ed25519::{EdwardsConfig, EdwardsProjective};

pub type Zp = <EdwardsConfig as CurveConfig>::ScalarField;
pub type G = EdwardsProjective;

pub trait IsZero {
    fn is_zero(&self) -> bool;
}

pub trait Block {
    // Encrypt in-place using AES256
    fn encrypt(&mut self, key: &[u8; 32]);

    fn decrypt(&mut self, key: &[u8; 32]);

    fn as_bytes(&self) -> &[u8];

    fn from_bytes(bytes: &[u8]) -> Self;
}

/// 128 bit data chunk
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block128([u8; 16]);

impl Block for Block128 {
    fn encrypt(&mut self, key: &[u8; 32]) {
        todo!()
    }

    fn decrypt(&mut self, key: &[u8; 32]) {
        todo!()
    }

    fn as_bytes(&self) -> &[u8] {
        todo!()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        todo!()
    }
}

impl IsZero for Block128 {
    fn is_zero(&self) -> bool {
        self.0.iter().all(|i| *i == 0)
    }
}

impl From<u128> for Block128 {
    fn from(value: u128) -> Self {
        Self(value.to_be_bytes())
    }
}

impl From<[u8; 16]> for Block128 {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block256([Block128; 2]);

impl Block for Block256 {
    fn encrypt(&mut self, key: &[u8; 32]) {
        todo!()
    }

    fn decrypt(&mut self, key: &[u8; 32]) {
        todo!()
    }

    fn as_bytes(&self) -> &[u8] {
        todo!()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        todo!()
    }
}

impl IsZero for Block256 {
    fn is_zero(&self) -> bool {
        self.0.iter().all(|i| i.is_zero())
    }
}

impl From<[Block128; 2]> for Block256 {
    fn from(value: [Block128; 2]) -> Self {
        Self(value)
    }
}

impl From<[u8; 32]> for Block256 {
    fn from(value: [u8; 32]) -> Self {
        let v0: [u8; 16] = value[0..16].try_into().unwrap();
        let v1: [u8; 16] = value[16..32].try_into().unwrap();
        Self([Block128::from(v0), Block128::from(v1)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_block256() {
        let val = [0u8; 32];

        let block = Block256::from(val);
        assert!(block.is_zero());
    }
}
