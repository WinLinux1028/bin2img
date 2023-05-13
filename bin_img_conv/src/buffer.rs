use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

#[derive(Clone)]
pub struct WritableArcRwLockVec(pub Rc<RefCell<Vec<u8>>>);

impl WritableArcRwLockVec {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }
}

impl Write for WritableArcRwLockVec {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut lock = self.0.borrow_mut();

        lock.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
