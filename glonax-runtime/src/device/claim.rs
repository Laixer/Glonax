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

    #[inline]
    pub fn as_path(&self) -> &Path {
        &self.node_path
    }
}
