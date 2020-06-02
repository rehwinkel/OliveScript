use super::super::codegen::Code;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub enum RefObject {
    Function {
        args: Vec<String>,
        codes: Vec<Code>,
    },
    String {
        value: String,
    },
    List {
        data: Vec<Garbage<Object>>,
    },
    Bendy {
        data: HashMap<String, Garbage<Object>>,
    },
    Native {
        arg_count: u32,
        closure: fn(Vec<Garbage<Object>>) -> Object,
    },
}

enum Object {
    Integer { value: i64 },
    Float { value: f64 },
    Boolean { value: bool },
    None,
    Pointer { value: Garbage<RefObject> },
}

pub struct Garbage<T> {
    data: *mut T,
    refcount: *mut usize,
}

impl<T: Sized> Garbage<T> {
    pub fn new(value: T) -> Self {
        let layout = Layout::new::<T>();
        let data;
        unsafe {
            data = alloc(layout) as *mut T;
            *data = value;
        }
        Garbage {
            data,
            refcount: Box::into_raw(Box::new(1)),
        }
    }
}

impl<T> Drop for Garbage<T> {
    fn drop(&mut self) {
        let layout = Layout::new::<T>();
        unsafe {
            dealloc(self.data as *mut u8, layout);
        }
    }
}

impl<T> Deref for Garbage<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.data.as_ref().unwrap() }
    }
}

impl<T> DerefMut for Garbage<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut().unwrap() }
    }
}

impl<T> Clone for Garbage<T> {
    fn clone(&self) -> Self {
        unsafe {
            *self.refcount += 1;
        }
        Garbage {
            data: self.data,
            refcount: self.refcount,
        }
    }
}
