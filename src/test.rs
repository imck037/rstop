#[cfg(test)]
mod tests {
    use crate::system;
    use crate::task;

    #[test]
    fn test_is_pid() {
        assert!(task::is_pid("1234"));
    }

    #[test]
    fn test_swap() {
        assert_eq!(system::get_swap(), Some((4194300, 0)));
    }
}
