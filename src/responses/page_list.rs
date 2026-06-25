pub(crate) trait PageList {
    fn next_url(&self) -> Option<&str>;
}
