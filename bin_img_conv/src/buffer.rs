use std::{
    io::{self, Write},
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct WritableArcRwLockVec(pub Arc<RwLock<Vec<u8>>>);

impl WritableArcRwLockVec {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Vec::new())))
    }
}

impl Write for WritableArcRwLockVec {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut lock = match self.0.write() {
            Ok(o) => o,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        };

        lock.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
