use olvnative::{Object, RuntimeError};
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! rc {
    ($e: expr) => {
        Rc::new(RefCell::new($e))
    };
}

#[no_mangle]
pub extern "C" fn n_sqrt(
    args: Box<Vec<Rc<RefCell<Object>>>>,
) -> Result<Rc<RefCell<Object>>, RuntimeError> {
    if let Object::Float(f) = &*args[0].borrow() {
        let val = Object::Float(f.sqrt());
        Ok(rc!(val))
    } else if let Object::Int(i) = &*args[0].borrow() {
        let val = Object::Float((*i as f64).sqrt());
        Ok(rc!(val))
    } else {
        Err(RuntimeError::TypeError)
    }
}
