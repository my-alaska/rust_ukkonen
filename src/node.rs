use std::collections::HashMap;

/// Index of a node in `UkkonenTree::nodes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NodeId(pub(crate) usize);

/// A node together with the edge leading into it from its parent.
/// The edge is labelled `sequence[label_start..label_end]`.
#[derive(Debug)]
pub(crate) struct Node<'a, T> {
    pub(crate) label_start: usize,

    /// Exclusive end of the label. `None` for leaves: their edge always runs
    /// to the current end of the sequence, so it grows as elements are added.
    pub(crate) label_end: Option<usize>,

    /// Outgoing edges, keyed by the first element of their label.
    pub(crate) children: HashMap<&'a T, NodeId>,

    /// For an internal node spelling `c` followed by `s`: the node spelling `s`.
    pub(crate) suffix_link: Option<NodeId>,
}

impl<T> Node<'_, T> {
    /// A node with no children and no suffix link.
    pub(crate) fn new(label_start: usize, label_end: Option<usize>) -> Self {
        Self {
            label_start,
            label_end,
            children: HashMap::new(),
            suffix_link: None,
        }
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}
