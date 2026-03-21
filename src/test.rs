#[cfg(test)]
mod tests {
    use crate::task;

    #[test]
    fn test_tasks() {
        assert_eq!(task::tasks(), (268, 0, 0, 0, 0, 0))
    }
    #[test]
    fn test_is_pid() {
        assert!(task::is_pid("1234"));
    }
}
