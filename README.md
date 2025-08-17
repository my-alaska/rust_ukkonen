# Ukkonen suffix tree creation algorithm in rust

---

### Contents

This repository is a tiny library I created as a simple rust exercise.

The implemented algorithm is Ukkonen's algorithm for linear building of suffix tree.

As for now the algorithm still requires adding special characters at the end of the sequence.
However, this is only necessary for the `.find()` method to work. 
The creation of the tree can be performed on-line by extending the tree structure with `.extend_tree()`.