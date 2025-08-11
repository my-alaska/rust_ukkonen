mod node;

use std::hash::Hash;
use crate::node::Node;

pub struct UkkonenTree<'a, T> {

    // internal tree representation
    root: Node<'a, T>,
}


impl<'a, T: Eq + Hash> UkkonenTree<'a, T> {
    pub fn new(sequence: &'a [T]) -> Self {
        let mut root = Node::new();
        root.start_index = Some(0);
        Self { root }
    }

    pub fn find(&self, pattern: &[T]) -> Option<Vec<(usize, usize)>> {
        None
    }
}