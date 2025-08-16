use std::collections::HashMap;
use std::hash::Hash;

use std::cell::RefCell;
use std::rc::{Rc};


pub struct Node<'a, T> {
    pub children: RefCell<HashMap<&'a T, Rc<Node<'a, T>>>>,
    pub special_link: RefCell<Option<Rc<Node<'a, T>>>>,
    pub start_idx: RefCell<usize>,
    pub end_idx: Option<usize>,
    pub leaf_index: Option<isize>,
}

impl<'a, T: Eq + Hash> Node<'a, T> {
    pub fn new(start_idx: usize) -> Self {
        Self {
            children: RefCell::new(HashMap::new()),
            special_link: RefCell::new(None),
            start_idx: RefCell::new(start_idx),
            end_idx: None,
            leaf_index: None,
        }
    }
}