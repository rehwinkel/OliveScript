use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

pub trait ReferenceHolder<T: ReferenceHolder<T> + PartialEq> {
    fn get_references(&self) -> Vec<Garbage<T>>;
}

pub struct Garbage<T: ReferenceHolder<T> + PartialEq> {
    data: *mut T,
}

impl<T: ReferenceHolder<T> + PartialEq> std::hash::Hash for Garbage<T> {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.data.hash(hasher);
    }
}

impl<T: ReferenceHolder<T> + PartialEq> PartialEq for Garbage<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.data == other.data {
            true
        } else {
            **self == **other
        }
    }
}

impl<T: ReferenceHolder<T> + PartialEq> Eq for Garbage<T> {}

/*
impl<T: ReferenceHolder<T>  + PartialEq> Debug for Garbage<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Garbage({:?})", unsafe { self.data.as_ref().unwrap() })
    }
}
*/

unsafe impl<T: ReferenceHolder<T> + PartialEq> Send for Garbage<T> {}

unsafe impl<T: ReferenceHolder<T> + PartialEq> Sync for Garbage<T> {}

impl<T: ReferenceHolder<T> + PartialEq> Deref for Garbage<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.data.as_ref().unwrap() }
    }
}

impl<T: ReferenceHolder<T> + PartialEq> DerefMut for Garbage<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut().unwrap() }
    }
}

impl<T: ReferenceHolder<T> + PartialEq> Clone for Garbage<T> {
    fn clone(&self) -> Self {
        Garbage { data: self.data }
    }
}

pub struct GarbageCollector<T: ReferenceHolder<T> + PartialEq> {
    references: HashSet<Garbage<T>>,
}

impl<T: ReferenceHolder<T> + PartialEq> GarbageCollector<T> {
    pub fn new() -> Self {
        GarbageCollector {
            references: HashSet::new(),
        }
    }

    pub fn alloc(&mut self, in_data: T) -> Garbage<T> {
        let data: *mut T = Box::into_raw(Box::new(in_data));
        let garbage = Garbage { data };
        self.references.insert(garbage.clone());
        garbage
    }

    fn dealloc(&mut self, obj: Garbage<T>) {
        unsafe {
            drop(Box::from_raw(obj.data));
        }
    }

    fn trace(&self, root: HashSet<Garbage<T>>) -> HashSet<Garbage<T>> {
        let mut reachable: HashSet<Garbage<T>> = HashSet::new();
        for reference in &root {
            let mut refshs = HashSet::new();
            refshs.extend(reference.get_references());
            reachable.extend(self.trace(refshs));
        }
        reachable.extend(root);
        reachable
    }

    pub fn run(&mut self, root: Vec<Garbage<T>>) {
        // println!("{}", self.references.len());
        let mut rooths = HashSet::new();
        rooths.extend(root);
        let reachable = self.trace(rooths);
        let dead: Vec<Garbage<T>> = self
            .references
            .iter()
            .filter(|r| !reachable.contains(r))
            .cloned()
            .collect();
        for dead_ref in dead {
            self.dealloc(dead_ref);
        }
    }
}
