/// Returns a random element from the given slice, caller should ensure that the slice is not empty.
pub fn rand_element<T>(v: &[T]) -> &T {
    debug_assert!(!v.is_empty());
    if v.len() == 1 {
        v.first().unwrap()
    } else {
        let idx = rand::random::<usize>() % v.len();
        v.get(idx).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand_element_should_work() {
        let v = vec![1, 2, 3];
        let e = rand_element(&v);
        assert!(v.contains(e));

        let v = vec![1];
        let &e = rand_element(&v);
        assert_eq!(1, e);

        let v = vec!["a", "b", "c"];
        let e = rand_element(&v);
        assert!(v.contains(e));
    }
}
