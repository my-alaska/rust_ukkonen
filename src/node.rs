use std::collections::HashMap;
use std::hash::Hash;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// Rc allows us to share the mutable RefCell
type NodeRef<'a, T> = Rc<RefCell<Node<'a, T>>>;
type WeakNodeRef<'a, T> = Weak<RefCell<Node<'a, T>>>;


pub struct Node<'a, T> {
    parent: Option<WeakNodeRef<'a, T>>,
    special_reference: Option<WeakNodeRef<'a, T>>,
    children: HashMap<&'a T, NodeRef<'a, T>>,
    pub start_index: Option<usize>,
}

impl<'a, T: Eq + Hash> Node<'a, T> {
    pub fn new() -> Self {
        Self {
            parent: None,
            special_reference: None,
            children: HashMap::new(),
            start_index: None,
        }
    }
}