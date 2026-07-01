/// Result of a successful link deletion.
#[derive(Debug)]
pub struct LinkDeleteResponse {
    /// Index of the interface that was deleted.
    pub if_index: u32,
}
