use std::path::{Path, PathBuf};

pub struct ResourceClaim {
    pub(super) node_path: PathBuf,
    pub(super) is_claimed: bool,
}

impl ResourceClaim {
    #[inline]
    pub(super) fn claim(&mut self) {
        self.is_claimed = true;
    }

    // TODO: Remove
    #[inline]
    pub fn is_claimed(&self) -> bool {
        self.is_claimed
    }

    pub fn as_path(&self) -> &Path {
        &self.node_path
    }
}
