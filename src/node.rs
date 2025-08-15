use std::collections::HashMap;
use std::hash::Hash;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// Rc allows us to share the mutable RefCell
type NodeRef<'a, T> = Rc<Node<'a, T>>;

// Breaks the cycle of references
type WeakNodeRef<'a, T> = Weak<Node<'a, T>>;


pub struct Node<'a, T> {
    pub children: RefCell<HashMap<&'a T, Rc<Node<'a, T>>>>,
    pub special_link: Option<Rc<Node<'a, T>>>,
    pub start_idx: usize,
    pub end_idx: Option<usize>,
    pub leaf_index: Option<isize>,
}

impl<'a, T: Eq + Hash> Node<'a, T> {
    pub fn new(start_idx: usize) -> Self {
        Self {
            children: RefCell::new(HashMap::new()),
            special_link: None,
            start_idx,
            end_idx: None,
            leaf_index: None,
        }
    }
}