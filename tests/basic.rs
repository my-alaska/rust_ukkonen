use std::collections::HashSet;
use rust_ukkonen::UkkonenTree;

#[test]
fn basic_string_search() {
    let text: Vec<char> = "babcababx".chars().collect();
    let tree = UkkonenTree::new(&text);
    tree.print_tree();

    let pattern: Vec<char> = "ab".chars().collect();

    let matches: HashSet<usize> = tree.find(&pattern).iter().copied().collect();
    assert_eq!(matches, HashSet::from([1, 4, 6]));

}

#[test]
fn missing_pattern() {
    let text: Vec<char> = "bananas".chars().collect();
    let tree = UkkonenTree::new(&text);

    let pattern: Vec<char> = "xyz".chars().collect();

    let matches: HashSet<usize> = tree.find(&pattern).iter().copied().collect();
    assert_eq!(matches, HashSet::new());
}

#[test]
fn works_on_numbers() {
    let nums = vec![1, 2, 3, 2, 3, 4];
    let tree = UkkonenTree::new(&nums);

    let matches: HashSet<usize> = tree.find(&[2, 3]).iter().copied().collect();

    assert_eq!(matches, HashSet::from([1, 3]));
}