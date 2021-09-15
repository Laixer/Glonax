use std::path::PathBuf;

pub struct Workspace {
    #[allow(dead_code)]
    db: sled::Db,
}

impl Workspace {
    /// Construct new workspace.
    ///
    /// The workspace is both a directory and a key value store.
    /// If the provided directory does not exist it will be created.
    pub fn new(path: &PathBuf) -> Self {
        Self::setup_if_not_exists(&path);

        debug!("Using workspace directory {}", path.to_str().unwrap());

        let db = sled::Config::default()
            .path(path)
            .flush_every_ms(Some(200))
            .open()
            .unwrap();

        Self { db }
    }

    /// Setup workspace directories if not exist.
    ///
    /// Thid method will create the absolte path.
    fn setup_if_not_exists(path: &PathBuf) {
        if !path.exists() {
            trace!("Workspace does not exit, creating one..");

            std::fs::create_dir_all(path).unwrap();
        }
    }
}
