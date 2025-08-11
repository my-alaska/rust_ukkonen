use rust_ukkonen::UkkonenTree;

#[test]
fn basic_string_search() {
    let text: Vec<char> = "banana".chars().collect();
    let st = UkkonenTree::new(&text);

    let pattern: Vec<char> = "ana".chars().collect();
    assert_eq!(st.find(&pattern), Some(vec![(1, 4), (3, 6)]));

}

#[test]
fn missing_pattern() {
    let text: Vec<char> = "banana".chars().collect();
    let st = UkkonenTree::new(&text);

    let pattern: Vec<char> = "xyz".chars().collect();
    assert_eq!(st.find(&pattern), None);
}

#[test]
fn works_on_numbers() {
    let nums = vec![1, 2, 3, 2, 3, 4];
    let st = UkkonenTree::new(&nums);
    assert_eq!(st.find(&[2, 3]), Some(vec![(1, 3), (3, 5)]));
}