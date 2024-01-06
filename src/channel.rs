use std::{
    cell::RefCell,
    io::{Error as IoError, Read, Write},
    rc::Rc,
};

pub trait AbstractChannel {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ChannelError>;

    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), ChannelError>;

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
    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), ChannelError> {
        self.reader.borrow_mut().read_exact(bytes)?;
        Ok(())
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
