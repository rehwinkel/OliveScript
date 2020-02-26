/*
#[no_mangle]
pub extern "C" fn n_sqrt(
    _args: Box<Vec<Rc<RefCell<Object>>>>,
) -> Result<Rc<RefCell<Object>>, RuntimeError> {
    Ok(Rc::new(RefCell::new(Object::Int(1234))))
}
*/
extern "C" void *n_sqrt(void *a)
{
    return a;
}