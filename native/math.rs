use std::cell::RefCell;
use std::rc::Rc;
extern crate olv;
use olv::{Object, RuntimeError};

#[no_mangle]
pub extern "C" fn n_sqrt(
    args: Vec<Rc<RefCell<Object>>>,
) -> Result<Rc<RefCell<Object>>, RuntimeError> {
    Ok(Rc::new(RefCell::new(Object::Int(1234))))
}
