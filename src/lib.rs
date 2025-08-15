mod node;

use std::hash::Hash;
use crate::node::Node;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// internal tree representation
pub struct UkkonenTree<'a, T> {

    // Nodes can have multiple references so we use Rc.
    root: Rc<Node<'a, T>>,
    active_node: Rc<Node<'a, T>>,

    seq_length: usize,
    remainder:usize,

    active_edge: Option<&'a T>, // flag that tells as what edge of the tree we're processing
    active_length: usize,
}


impl<'a, T: Eq + Hash> UkkonenTree<'a, T> {
    pub fn new(sequence: &'a [T]) -> Self {

        // Create a root node
        let mut root = Node::new(0);
        root.leaf_index = Some(0);
        let root = Rc::new(root);

        // Initialise the tree object
        let mut tree = Self {
            root: root.clone(),
            active_node: root,
            seq_length: 0 ,
            remainder: 0,
            active_edge: None,
            active_length: 0,
        };

        // We're "streaming" the data into the tree
        for element in sequence {
            tree.extend_tree(element);
        }

        tree
    }

    fn walk_down(&mut self) -> bool {
        // TODO
        true
    }

    // This method can be used to extend existing tree with new elements
    pub fn extend_tree(&mut self, element: &'a T) {
        // The length of our sequence is being extended by the online algorithm
        self.seq_length += 1;

        // initialize the checker for the most recently created node
        let mut last_created_node : Option<Node<T>> = None;

        // Since there is one new element, we increase remainder by one
        // It's the number of characters to process or implicitly stored
        self.remainder += 1;
        while self.remainder > 0 {

            // If the active node doesn't have edge corresponding to our letter
            if self.active_length == 0 {

                // If active length is 0 we select active edge
                self.active_edge = Some(element);

                // If active node has no edge corresponding to `element`...
                if !self.active_node.children.borrow().contains_key(element)  {
                    // create a new child node and edge that does

                    // initialize node
                    let new_node = Node::new(self.seq_length - 1);
                    let new_node = Rc::new(new_node);

                    // and input it as a child into the active node.
                    // We need to borrow a mutable reference to the children hashmap
                    self.active_node.children.borrow_mut().insert(element, new_node);
                }

                // If new node was created in previous iterations
                // create a special link from it to current active node
                if !Rc::ptr_eq(&self.root, &self.active_node){
                    if let Some(mut new_node) = last_created_node.take(){
                        new_node.special_link = Some(Rc::clone(&self.active_node));
                        last_created_node = None;
                    }
                }

            // If there is active length
            } else {
                // check if we can "walk down" - active length is greater than the length of edge below
                // If we walk down we have to process the beginning of the loop
                if self.walk_down() { continue }

                // extract the child at the end of the active edge. It has some important information
                let children_ref = self.active_node.children.borrow_mut();
                let mut active_child = children_ref.get(element).unwrap();

                // In case our new letter is already implicitly represented on the active edge
                // We want to update active length (and create a link if need be)
                // We break out of the loop here. It's the default case of implicit extension
                let new_node: Node<T> = Node::new(active_child.start_idx);
                // todo create the new_node

                // check if there was already a new node created in this tree extension step.
                // If it was, we have to link the new node to it
                if let Some(mut new_node) = last_created_node.take() {
                    new_node.special_link = Some(Rc::clone(&self.active_node));
                    last_created_node = Some(new_node);
                }
            }

            self.remainder -= 1;

            // If root is the active node at this point it means we inserted a new node or leaf from it

            // If root is the active edge, and the active length is nonzero:
            if !Rc::ptr_eq(&self.root, &self.active_node) {
                // change the active edge and lower the active length

                self.active_length -= 1;
                // todo use text
                // self.active_edge = text[self.seq_length - self.remainder]
            }

            // Update current active node with links or reset it to root otherwise
            if let Some(linked_node) = &self.active_node.special_link {
                self.active_node = Rc::clone(&linked_node);
            } else {
                self.active_node = Rc::clone(&self.root);
            }
        }

    }

    pub fn find(&self, pattern: &[T]) -> Option<Vec<(usize, usize)>> {
        // todo
        None
    }
}