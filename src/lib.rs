//! Suffix tree built on-line with Ukkonen's algorithm.
//!
//! ```
//! use rust_ukkonen::UkkonenTree;
//!
//! let text: Vec<char> = "banana".chars().collect();
//! let tree = UkkonenTree::new(&text);
//!
//! let pattern: Vec<char> = "ana".chars().collect();
//! assert_eq!(tree.find(&pattern), vec![1, 3]);
//! ```
//! The algorithm reads the sequence from the start and keeps the suffix tree of
//! the prefix read so far:
//!
//! 1. **Open leaf edges.** A leaf edge ends at the end of the sequence, so all
//!    leaves grow by one element for free.
//! 2. **Active point and remainder.** A suffix that occurs earlier in the
//!    sequence is *implicit*: it ends inside an edge, not at a leaf. `remainder`
//!    counts them and the *active point* marks where the longest one ends.
//! 3. **Suffix links.** The internal node spelling `c` followed by `s` links to
//!    the node spelling `s`, so moving to the next shorter suffix is a jump,
//!    not a walk from the root.

mod node;

use std::fmt::{self, Display};
use std::hash::Hash;

use crate::node::{Node, NodeId};

/// The root is always the first node pushed into the arena.
const ROOT: NodeId = NodeId(0);

/// Suffix tree of a sequence. Borrows the elements instead of copying them.
#[derive(Debug)]
pub struct UkkonenTree<'a, T> {
    /// Arena of nodes; nodes refer to each other by index.
    nodes: Vec<Node<'a, T>>,

    /// Elements read so far. Edge labels are ranges into this sequence.
    sequence: Vec<&'a T>,

    // Active point: where the longest implicit suffix ends. It lies
    // `active_length` elements down the edge out of `active_node` whose label
    // starts with `sequence[active_edge]`.
    active_node: NodeId,
    active_edge: usize,
    active_length: usize,

    /// Number of suffixes that are still implicit (have no leaf yet).
    remainder: usize,
}

impl<'a, T: Eq + Hash> UkkonenTree<'a, T> {
    /// Builds the suffix tree of `sequence`, one element at a time.
    #[must_use]
    pub fn new(sequence: &'a [T]) -> Self {
        let mut tree: UkkonenTree<'_, T> = Self {
            // The root has no incoming edge, so its label is empty.
            nodes: vec![Node::new(0, Some(0))],
            sequence: Vec::with_capacity(sequence.len()),
            active_node: ROOT,
            active_edge: 0,
            active_length: 0,
            remainder: 0,
        };

        // We're "streaming" the data into the tree, one element (phase) at a time.
        tree.extend(sequence);
        tree
    }

    /// Appends one element: one phase of the algorithm, amortised O(1).
    pub fn extend_tree(&mut self, element: &'a T) {
        let new_pos: usize = self.sequence.len();
        self.sequence.push(element);
        self.remainder += 1;

        // Internal node created earlier in this phase whose suffix link is not set yet.
        let mut awaiting_link: Option<NodeId> = None;

        while self.remainder > 0 {
            // Active point sits on a node: look for an edge starting with the new element.
            if self.active_length == 0 {
                self.active_edge = new_pos;
            }

            let edge_first: &T = self.sequence[self.active_edge];

            // Check if the active edge exists: either no edge out of the active node
            // starts with `edge_first` (None), or there is one leading to another node (Some).
            match self
                .node(self.active_node)
                .children
                .get(edge_first)
                .copied()
            {
                None => {
                    // No such edge: the suffix ends at the active node, so add a leaf there.
                    let leaf: NodeId = self.add_node(Node::new(new_pos, None));
                    self.node_mut(self.active_node)
                        .children
                        .insert(element, leaf);
                    self.set_suffix_link(awaiting_link.take(), self.active_node);
                }
                Some(child) => {
                    // Skip/count: the active point is past this edge, so move down to `child`.
                    let label_len: usize = self.label_len(child);
                    if self.active_length >= label_len {
                        self.active_edge += label_len;
                        self.active_length -= label_len;
                        self.active_node = child;
                        continue;
                    }

                    let label_start: usize = self.node(child).label_start;
                    if self.sequence[label_start + self.active_length] == element {
                        // The element is already on the edge: this suffix and all
                        // shorter ones stay implicit, so the phase ends.
                        self.set_suffix_link(awaiting_link.take(), self.active_node);
                        self.active_length += 1;
                        break;
                    }

                    // Mismatch inside the edge: split it at the active point and add a leaf.
                    //
                    //   active_node --[label_start..split_pos)--> split --[split_pos..)--> child
                    //                                                   \--[new_pos..)----> leaf
                    let split_pos: usize = label_start + self.active_length;
                    let split: NodeId = self.add_node(Node::new(label_start, Some(split_pos)));
                    let leaf: NodeId = self.add_node(Node::new(new_pos, None));

                    // Child keys are the first elements of the new edge labels.
                    let split_key: &T = self.sequence[label_start];
                    let child_key: &T = self.sequence[split_pos];

                    let split_node: &mut Node<'_, T> = self.node_mut(split);
                    split_node.children.insert(element, leaf);
                    split_node.children.insert(child_key, child);
                    self.node_mut(child).label_start = split_pos;
                    self.node_mut(self.active_node)
                        .children
                        .insert(split_key, split);

                    // Link the previous waiting node to `split`; `split` now waits for its own link.
                    self.set_suffix_link(awaiting_link.replace(split), split);
                }
            }

            // The suffix now ends at a leaf.
            self.remainder -= 1;

            // Move the active point to the next shorter suffix: at the root, drop
            // its first element; elsewhere, follow the suffix link.
            if self.active_node == ROOT {
                if self.active_length > 0 {
                    self.active_length -= 1;
                    self.active_edge = new_pos + 1 - self.remainder;
                }
            } else {
                self.active_node = self.node(self.active_node).suffix_link.unwrap_or(ROOT);
            }
        }
    }

    /// Positions of all occurrences of `pattern`, in ascending order.
    #[must_use]
    pub fn find(&self, pattern: &[T]) -> Vec<usize> {
        let Some((node, depth)) = self.locate(pattern) else {
            return Vec::new();
        };

        // Each leaf below the node is a suffix starting with `pattern`.
        // `depth` counts the elements on the path from the root, so a leaf at
        // depth d is the suffix starting at `sequence.len() - d`.
        let mut positions: Vec<usize> = Vec::new();
        let mut stack = vec![(node, depth)];
        while let Some((id, depth)) = stack.pop() {
            if self.node(id).is_leaf() {
                // A root without children means an empty tree.
                if id != ROOT {
                    positions.push(self.sequence.len() - depth);
                }
            } else {
                stack.extend(
                    self.node(id)
                        .children
                        .values()
                        .map(|&child| (child, depth + self.label_len(child))),
                );
            }
        }
        positions.sort_unstable();

        // Implicit suffixes have no leaf. They are the last `remainder`
        // suffixes, so their matches are the occurrences inside that tail.
        let implicit_start: usize = self.sequence.len() - self.remainder;
        let tail: &[&T] = &self.sequence[implicit_start..];
        positions.extend(
            kmp_find(tail, pattern)
                .into_iter()
                .map(|offset| implicit_start + offset),
        );
        positions
    }

    /// Follows `pattern` down from the root. Returns the node at or just below
    /// its end and that node's depth, or `None` if `pattern` does not occur.
    fn locate(&self, pattern: &[T]) -> Option<(NodeId, usize)> {
        let mut node: NodeId = ROOT;
        let mut depth: usize = 0;
        let mut matched: usize = 0;

        while let Some(next_element) = pattern.get(matched) {
            let child: NodeId = *self.node(node).children.get(next_element)?;
            let label_start: usize = self.node(child).label_start;
            let label_len: usize = self.label_len(child);
            // The pattern may end partway along this edge.
            let compared: usize = label_len.min(pattern.len() - matched);

            let label: &[&T] = &self.sequence[label_start..label_start + compared];
            let rest: &[T] = &pattern[matched..matched + compared];
            if !label.iter().copied().eq(rest) {
                return None;
            }

            node = child;
            depth += label_len;
            matched += compared;
        }
        Some((node, depth))
    }

    /// Sets the suffix link of `from` to `to`; does nothing if `from` is `None`.
    ///
    /// A new internal node only learns its link target when the algorithm
    /// reaches the next shorter suffix, so `from` is the node waiting for it.
    fn set_suffix_link(&mut self, from: Option<NodeId>, to: NodeId) {
        if let Some(from) = from {
            self.node_mut(from).suffix_link = Some(to);
        }
    }
}

impl<'a, T> UkkonenTree<'a, T> {
    fn node(&self, id: NodeId) -> &Node<'a, T> {
        &self.nodes[id.0]
    }

    fn node_mut(&mut self, id: NodeId) -> &mut Node<'a, T> {
        &mut self.nodes[id.0]
    }

    /// Stores `node` in the arena and returns its ID.
    fn add_node(&mut self, node: Node<'a, T>) -> NodeId {
        self.nodes.push(node);
        NodeId(self.nodes.len() - 1)
    }

    /// Exclusive end of the label of the edge into `id`; for a leaf, the
    /// current end of the sequence.
    fn label_end(&self, id: NodeId) -> usize {
        self.node(id).label_end.unwrap_or(self.sequence.len())
    }

    /// Number of elements on the edge into `id`.
    fn label_len(&self, id: NodeId) -> usize {
        self.label_end(id) - self.node(id).label_start
    }
}

impl<T: Display> UkkonenTree<'_, T> {
    /// Prints the tree to stdout in the format of the `Display` impl.
    pub fn print_tree(&self) {
        print!("{self}");
    }
}

/// Appends the elements one by one, as [`UkkonenTree::extend_tree`] does.
impl<'a, T: Eq + Hash> Extend<&'a T> for UkkonenTree<'a, T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for element in iter {
            self.extend_tree(element);
        }
    }
}

/// One edge label per line, indented by the edge's depth in the tree.
impl<T: Display> Display for UkkonenTree<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stack: Vec<(NodeId, usize)> = self
            .node(ROOT)
            .children
            .values()
            .map(|&id| (id, 0))
            .collect();
        while let Some((id, level)) = stack.pop() {
            write!(f, "{}", "    ".repeat(level))?;
            for element in &self.sequence[self.node(id).label_start..self.label_end(id)] {
                write!(f, "{element}")?;
            }
            writeln!(f)?;
            stack.extend(
                self.node(id)
                    .children
                    .values()
                    .map(|&child| (child, level + 1)),
            );
        }
        Ok(())
    }
}

/// Start offsets of `pattern` in `text` (Knuth-Morris-Pratt).
fn kmp_find<T: Eq>(text: &[&T], pattern: &[T]) -> Vec<usize> {
    if pattern.is_empty() {
        return (0..text.len()).collect();
    }

    // border[i]: length of the longest proper prefix of `pattern[..=i]`
    // that is also its suffix.
    let mut border = vec![0; pattern.len()];
    let mut matched: usize = 0;
    for i in 1..pattern.len() {
        while matched > 0 && pattern[i] != pattern[matched] {
            matched = border[matched - 1];
        }
        if pattern[i] == pattern[matched] {
            matched += 1;
        }
        border[i] = matched;
    }

    // Scan `text`; on a mismatch, fall back along `border` instead of restarting.
    let mut offsets = Vec::new();
    let mut matched = 0;
    for (i, &element) in text.iter().enumerate() {
        while matched > 0 && *element != pattern[matched] {
            matched = border[matched - 1];
        }
        if *element == pattern[matched] {
            matched += 1;
        }
        if matched == pattern.len() {
            offsets.push(i + 1 - matched);
            matched = border[matched - 1];
        }
    }
    offsets
}
