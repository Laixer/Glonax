use std::path::{Path, PathBuf};

pub struct ResourceClaim {
    #[allow(dead_code)]
    sys_path: PathBuf,
    node_path: PathBuf,
    is_claimed: bool,
}

impl ResourceClaim {
    pub(super) fn new(sys_path: &Path, node_path: &Path) -> Self {
        Self {
            sys_path: sys_path.to_path_buf(),
            node_path: node_path.to_path_buf(),
            is_claimed: false,
        }
    }

    #[inline]
    pub(super) fn claim(&mut self) {
        self.is_claimed = true;
    }

    #[inline]
    pub fn is_claimed(&self) -> bool {
        self.is_claimed
    }

    pub fn as_path(&self) -> &Path {
        &self.node_path
    }
}
