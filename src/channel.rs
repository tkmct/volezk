use std::{
    cell::RefCell,
    io::{Error as IoError, Read, Write},
    rc::Rc,
};

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};

use crate::types::*;

pub trait AbstractChannel {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ChannelError>;

    fn write_zp(&mut self, val: Zp) -> Result<(), ChannelError>;

    fn write_g(&mut self, val: G) -> Result<(), ChannelError>;

    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), ChannelError>;

    fn read_zp(&mut self) -> Result<Zp, ChannelError>;

    fn read_g(&mut self) -> Result<G, ChannelError>;

    fn flush(&mut self) -> Result<(), ChannelError>;

    fn clone(&self) -> Self;
}

pub struct Channel<R, W> {
    reader: Rc<RefCell<R>>,
    writer: Rc<RefCell<W>>,
}

#[derive(thiserror::Error, Debug)]
pub enum ChannelError {
    #[error(transparent)]
    Io {
        #[from]
        source: IoError,
    },
    #[error(transparent)]
    Serialize {
        #[from]
        source: SerializationError,
    },
}

impl<R: Read, W: Write> Channel<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: Rc::new(RefCell::new(reader)),
            writer: Rc::new(RefCell::new(writer)),
        }
    }

    pub fn reader(self) -> Rc<RefCell<R>> {
        self.reader
    }

    pub fn writer(self) -> Rc<RefCell<W>> {
        self.writer
    }
}

impl<R: Read, W: Write> AbstractChannel for Channel<R, W> {
    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ChannelError> {
        self.writer.borrow_mut().write_all(bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn write_zp(&mut self, val: Zp) -> Result<(), ChannelError> {
        let mut buff = Vec::new();
        val.serialize_compressed(&mut buff)?;
        self.write_bytes(&buff)
    }

    #[inline(always)]
    fn write_g(&mut self, val: G) -> Result<(), ChannelError> {
        let mut buff = Vec::new();
        val.serialize_compressed(&mut buff)?;
        self.write_bytes(&buff)
    }

    #[inline(always)]
    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), ChannelError> {
        self.reader.borrow_mut().read_exact(bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn read_zp(&mut self) -> Result<Zp, ChannelError> {
        let mut buff = vec![0; 32];
        self.read_bytes(&mut buff)?;
        let val = Zp::deserialize_compressed(&*buff)?;
        Ok(val)
    }

    #[inline(always)]
    fn read_g(&mut self) -> Result<G, ChannelError> {
        let mut buff = vec![0; 32];
        self.read_bytes(&mut buff)?;
        let val = G::deserialize_compressed(&*buff)?;
        Ok(val)
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), ChannelError> {
        self.writer.borrow_mut().flush()?;
        Ok(())
    }

    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
            writer: self.writer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::UniformRand;
    use rand::prelude::thread_rng;
    use std::os::unix::net::UnixStream;

    #[test]
    fn test_read_zp() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let mut sender_channel = Channel::new(sender.try_clone().unwrap(), sender);
        let mut receiver_channel = Channel::new(receiver.try_clone().unwrap(), receiver);

        let mut rng = thread_rng();
        let val = Zp::rand(&mut rng);
        sender_channel.write_zp(val).unwrap();

        let result = receiver_channel.read_zp().unwrap();

        assert_eq!(val, result);
    }

    #[test]
    fn test_read_g() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let mut sender_channel = Channel::new(sender.try_clone().unwrap(), sender);
        let mut receiver_channel = Channel::new(receiver.try_clone().unwrap(), receiver);

        let mut rng = thread_rng();
        let val = G::rand(&mut rng);
        sender_channel.write_g(val).unwrap();

        let result = receiver_channel.read_g().unwrap();

        assert_eq!(val, result);
    }
}
