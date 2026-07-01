/// Result of a successful link flag update.
#[derive(Debug)]
pub struct LinkSetResponse {
    /// Index of the interface that was modified.
    pub if_index: u32,
}
