use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes256;
use std::default::Default;

use crate::types::IsZero;

#[derive(thiserror::Error, Debug)]
enum BlockError {}

pub trait Block {
    // Encrypt in-place using AES256
    fn encrypt(&self, key: &[u8; 32]) -> Self;

    fn decrypt(&self, key: &[u8; 32]) -> Self;

    fn as_bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &[u8]) -> Self;

    fn bytes_len(&self) -> usize;
}

/// 128 bit data chunk
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Block128([u8; 16]);

impl Block for Block128 {
    fn encrypt(&self, key: &[u8; 32]) -> Self {
        let g_key = GenericArray::from(*key);
        let cipher = Aes256::new(&g_key);

        let mut g_val = GenericArray::from(self.0);
        cipher.encrypt_block(&mut g_val);

        Self(g_val.try_into().unwrap())
    }

    fn decrypt(&self, key: &[u8; 32]) -> Self {
        let g_key = GenericArray::from(*key);
        let cipher = Aes256::new(&g_key);

        let mut g_val = GenericArray::from(self.0);
        cipher.decrypt_block(&mut g_val);

        Self(g_val.try_into().unwrap())
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Convert slice of byte into 128bit chunked block.
    /// If the length of the input is larger than 16, it uses the first 128bit and construct Block.
    /// If the length of the input is less than 16, the block will be padded with zeros.
    fn from_bytes(bytes: &[u8]) -> Self {
        // check input length
        if bytes.len() < 16 {
            let mut dst = [0u8; 16];
            dst[..bytes.len()].copy_from_slice(bytes);
            Self(dst)
        } else {
            Self(bytes[0..16].try_into().unwrap())
        }
    }

    fn bytes_len(&self) -> usize {
        16
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Block256([Block128; 2]);

impl Block for Block256 {
    fn encrypt(&self, key: &[u8; 32]) -> Self {
        let b_0 = self.0[0].encrypt(key);
        let b_1 = self.0[1].encrypt(key);
        Self([b_0, b_1])
    }

    fn decrypt(&self, key: &[u8; 32]) -> Self {
        let b_0 = self.0[0].decrypt(key);
        let b_1 = self.0[1].decrypt(key);
        Self([b_0, b_1])
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.0
            .iter()
            .map(|b| b.as_bytes())
            .collect::<Vec<_>>()
            .concat()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        // check input length
        if bytes.len() < 32 {
            let mut dst = [0u8; 32];
            dst[..bytes.len()].copy_from_slice(bytes);
            Self([
                Block128::from_bytes(&dst[0..16]),
                Block128::from_bytes(&dst[16..32]),
            ])
        } else {
            Self([
                Block128::from_bytes(&bytes[0..16]),
                Block128::from_bytes(&bytes[16..32]),
            ])
        }
    }

    fn bytes_len(&self) -> usize {
        32
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

// Impl arbitrary length Block
impl Block for Vec<Block128> {
    fn encrypt(&self, key: &[u8; 32]) -> Self {
        self.iter().map(|b| b.encrypt(key)).collect::<Vec<_>>()
    }

    fn decrypt(&self, key: &[u8; 32]) -> Self {
        self.iter().map(|b| b.decrypt(key)).collect::<Vec<_>>()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|b| b.as_bytes()).collect::<Vec<_>>()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        bytes
            .chunks(16)
            .map(Block128::from_bytes)
            .collect::<Vec<_>>()
    }

    fn bytes_len(&self) -> usize {
        self.len() * 16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::{Digest, Keccak256};

    fn gen_key() -> [u8; 32] {
        let mut hasher = Keccak256::default();
        hasher.update(b"Key");
        hasher.finalize().try_into().unwrap()
    }

    #[test]
    fn test_cast_block256() {
        let block = Block256::from([0u8; 32]);
        assert!(block.is_zero());
    }

    #[test]
    fn test_block128_from_bytes() {
        let block = Block128::from_bytes(&[1u8; 12]);
        assert_eq!(block.0, [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,]);
    }

    #[test]
    fn test_block128_from_bytes_2() {
        let val = [1u8; 18];
        let block = Block128::from_bytes(&val);
        assert_eq!(block.0, [1u8; 16]);
    }

    #[test]
    fn test_block128_encrypt_decrypt() {
        let block = Block128::from_bytes(&[1u8; 16]);
        let key = gen_key();

        let encrypted = block.encrypt(&key);
        let decrypted = encrypted.decrypt(&key);

        assert_eq!(decrypted, block);
    }

    #[test]
    fn test_block_vec() {
        let blocks = vec![
            Block128::from_bytes(&[1u8; 16]),
            Block128::from_bytes(&[0u8; 16]),
        ];

        let key = gen_key();
        let encrypted = blocks.encrypt(&key);
        let decrypted = encrypted.decrypt(&key);

        assert_eq!(decrypted, blocks);
    }
}
