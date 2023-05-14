use std::{
    cell::RefCell,
    cmp::min,
    io::{self, Read, Write},
    rc::Rc,
};

pub struct LowMemoryReadableVec(Vec<u8>);

impl From<Vec<u8>> for LowMemoryReadableVec {
    fn from(mut input: Vec<u8>) -> Self {
        input.reverse();
        LowMemoryReadableVec(input)
    }
}

impl Read for LowMemoryReadableVec {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = min(self.0.len(), buf.len());
        for i in 0..result {
            unsafe {
                *buf.get_unchecked_mut(i) = self.0.pop().unwrap_unchecked();
            }
        }
        self.0.shrink_to_fit();

        Ok(result)
    }
}

#[derive(Clone)]
pub struct WritableRcRefCellVec(pub Rc<RefCell<Vec<u8>>>);

impl WritableRcRefCellVec {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }
}

impl Write for WritableRcRefCellVec {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut lock = self.0.borrow_mut();

        lock.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
