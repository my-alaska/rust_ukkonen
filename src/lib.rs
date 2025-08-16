mod node;

use std::fmt::Display;
use std::hash::Hash;
use crate::node::Node;

use std::rc::{Rc};

// internal tree representation
pub struct UkkonenTree<'a, T> {

    // Nodes can have multiple references so we use Rc.
    root: Rc<Node<'a, T>>,
    active_node: Rc<Node<'a, T>>,

    remainder:usize,

    active_edge: Option<usize>, // flag that tells as what edge of the tree we're processing
    active_length: usize,

    sequence: Vec<&'a T>,
}


impl<'a, T: Eq + Hash + Display> UkkonenTree<'a, T> {
    pub fn new(sequence: &'a [T]) -> Self {

        // Create a root node
        let mut root = Node::new(0);
        root.leaf_index = Some(0);
        let root = Rc::new(root);

        // Initialise the tree object
        let mut tree = Self {
            root: root.clone(),
            active_node: root,
            remainder: 0,
            active_edge: None,
            active_length: 0,
            sequence: Vec::new(),
        };

        // We're "streaming" the data into the tree
        for element in sequence {
            tree.extend_tree(&element);
        }

        tree
    }

    fn walk_down(&mut self) -> bool {
        println!("    walking down");
        let element = self.sequence[self.active_edge.unwrap()];

        let children = self.active_node.children.borrow();

        if !children.contains_key(element) {

            println!("    walk down failed - no child in keys");
            return false;
        }

        let active_child = Rc::clone(children.get(&element).unwrap());
        // Drop the borrow to children explicitly.
        // Otherwise compiler won't let us assign the active node
        drop(children);

        if let Some(end_idx) = active_child.end_idx{
            let edge_length = end_idx - *active_child.start_idx.borrow();
            if self.active_length >= edge_length {

                self.active_edge = Some(self.active_edge.unwrap() + edge_length);
                self.active_length -= edge_length;
                self.active_node = active_child;

                println!("    walk down successful");
                return true
            }
            println!("    walk down failed - leaf node");
        }

        println!("    walk down failed");
        false
    }

    // This function can be used to link the last_created_node to active node
    // We do this check in a few places during tree extension
    fn link_created_node(&self, last_created_node: Option<Rc<Node<'a, T>>>, node_to_link: Rc<Node<'a, T>>) -> Option<Rc<Node<'a, T>>> {
        last_created_node.map(|new_node| {
            new_node.special_link.replace(Some(node_to_link));
            new_node
        })
    }

    fn update_active_node(&mut self) {
        let linked_node = &self.active_node.special_link.borrow_mut().take();

        if let Some(linked_node) = linked_node {
            self.active_node = Rc::clone(&linked_node);
        } else {
            self.active_node = Rc::clone(&self.root);
        }
    }

    // This method can be used to extend existing tree with new elements
    pub fn extend_tree(&mut self, element: &'a T) {
        println!();
        println!("creating node for element {}", element);

        self.sequence.push(element);

        // initialize the checker for the most recently created node
        let mut last_created_node : Option<Rc<Node<T>>> = None;

        // Since there is one new element, we increase remainder by one
        // It's the number of characters to process or implicitly stored
        self.remainder += 1;
        while self.remainder > 0 {
            println!("starting loop with: ");
            if let Some(active_edge) = self.active_edge {
                println!("    active edge: {} ", self.sequence[active_edge]);
            } else {
                println!("    active edge: None ");
            }
            println!("    remainder: {}", self.remainder);
            println!("    active length: {}",self.active_length);


            if self.active_length == 0{
                // If active length is 0 we select active edge
                self.active_edge = Some(self.sequence.len() - 1);


                println!("    active length is zero!");
                println!("    active edge updated to {}", self.sequence[self.active_edge.unwrap()]);
            }

            // If the active node doesn't have edge corresponding to our letter
            if self.active_length == 0 && !self.active_node.children.borrow().contains_key(element){
                println!("    element NOT found in active node keys");
                // create a new child node and edge that does

                // initialize node
                let new_node = Node::new(self.sequence.len() - 1);
                let new_node = Rc::new(new_node);

                // and input it as a child into the active node.
                // We need to borrow a mutable reference to the children hashmap
                self.active_node.children.borrow_mut().insert(element, new_node);


                // If new node was created in previous iterations
                // create a special link from it to current active node
                if !Rc::ptr_eq(&self.root, &self.active_node){
                    println!("    creating a special link");

                    self.link_created_node(last_created_node, self.active_node.clone());
                    last_created_node = None;
                }

            // If there is active length
            } else {
                println!("    element found in active node keys");

                // check if we can "walk down" - active length is greater than the length of edge below
                // If we walk down we have to process the beginning of the loop
                if self.walk_down() { continue }

                // extract the child at the end of the active edge. It has some important information
                let mut children_ref = self.active_node.children.borrow_mut();
                let active_child = children_ref.get(element).unwrap();

                // In case our new letter is already implicitly represented on the active edge
                // We want to update active length (and create a link if need be)
                // We break out of the loop here. It's the default case of implicit extension
                if self.sequence[*active_child.start_idx.borrow() + self.active_length] == element {
                    println!("    active sequence extended implicitly");

                    // Update link of last_created_node
                    self.link_created_node(last_created_node,self.active_node.clone());

                    self.active_length += 1;
                    break;
                }

                // Otherwise we have to break the path
                println!("    breaking the path!");

                // create a new node with proper beginning and ending index
                let new_node_start = *active_child.start_idx.borrow();
                let new_node_end = new_node_start + self.active_length;
                let mut break_path_node: Node<T> = Node::new(new_node_start);
                break_path_node.end_idx = Some(new_node_end);

                // Modify active child start index
                active_child.start_idx.replace(new_node_end);
                // Create a new child
                let new_node = Rc::new(Node::new(self.sequence.len() - 1));

                let mut break_path_children = break_path_node.children.borrow_mut();
                break_path_children.insert(self.sequence[break_path_node.end_idx.unwrap()], Rc::clone(&active_child));
                break_path_children.insert(element, new_node);
                drop(break_path_children);

                // Assign the new node as child of active_node
                let break_path_node = Rc::new(break_path_node);
                children_ref.insert(self.sequence[new_node_start], Rc::clone(&break_path_node));

                // check if there was already a new node created in this tree extension step.
                // If it was, we have to link the new node to it
                self.link_created_node(last_created_node, Rc::clone(&break_path_node));
                last_created_node = Some(break_path_node);
            }
            drop(self.active_node.children.borrow_mut());

            println!("    outside of `if`");

            self.remainder -= 1;

            // If root is the active node at this point it means we inserted a new node or leaf from it

            // If root is the active edge, and the active length is nonzero:
            if Rc::ptr_eq(&self.root, &self.active_node) && self.active_length > 0{
                println!("    lowering the remainder");

                // Select the active edge on the root using the 2nd character of active sequence
                self.active_edge = Some(self.sequence.len() - self.remainder);

                // since active sequence is now shorter we decrease active length
                self.active_length -= 1;
            }

            // Update current active node with links or reset it to root
            self.update_active_node();
        }
    }

    fn dfs(&self, node: Rc<Node<'a, T>>, result: &mut Vec<usize>){
        if node.children.borrow().len() == 0 {
            result.push(node.start_idx.borrow().clone());
        } else {
            for child in node.children.borrow().values() {
                self.dfs(Rc::clone(child), result);
            }
        }
    }

    fn bfs_print(&self, node: Rc<Node<'a, T>>, depth: usize) {
        let idx = node.start_idx.borrow().clone();
        println!("{}{} (idx {})", " ".repeat(depth*4), self.sequence[idx], idx);

        for child in node.children.borrow().values() {
            self.bfs_print(Rc::clone(child), depth + 1);
        }
    }

    pub fn find(&self, pattern: &[T]) -> Vec<usize> {
        let mut result = Vec::new();

        let mut node = Rc::clone(&self.root);
        let mut pattern_start = 0;

        self.bfs_print(Rc::clone(&node), 0);

        loop {
            // println!("{}", pattern_start);
            let child = {
                let children = node.children.borrow();
                // for child in children.values() {
                //     print!("{} ", self.sequence[child.start_idx.borrow().clone()]);
                // }
                if let Some(child) = children.get(pattern.get(pattern_start).unwrap()) {
                    Some(Rc::clone(child))
                } else {
                    None
                }
            };

            if let None = child { break; }

            let child = child.unwrap();

            let seq_start = child.start_idx.borrow().clone();


            let seq_end = match child.end_idx {
                Some(end) => end,
                None => self.sequence.len()
            };

            let seq_len = seq_end - seq_start;

            let (pattern_end, walk_down) = {
                if pattern_start + seq_len < pattern.len() {
                    (seq_start + seq_len, true)
                } else {
                    (pattern.len(), false)
                }
            };

            let subseq = self.sequence[seq_start..seq_end].iter().copied();
            let sub_pattern = pattern[pattern_start..pattern_end].iter();

            if subseq.eq(sub_pattern) {
                if walk_down {
                    node = child;
                    pattern_start += seq_len;
                    continue
                }
                self.dfs(child, &mut result);
            }

            break;

        }
        println!("");
        result
    }
}