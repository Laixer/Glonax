pub struct Workspace {
    #[allow(dead_code)]
    db: sled::Db,
}

impl Workspace {
    /// Construct new workspace.
    pub fn new(path: &std::path::PathBuf) -> Self {
        Self::setup_if_not_exists(&path);

        debug!("Using workspace directory {}", path.to_str().unwrap());

        let db = sled::Config::default()
            .path(path)
            .flush_every_ms(Some(200))
            .open()
            .unwrap();

        db.insert("last_boot", &12u32.to_be_bytes()).unwrap();
        Self { db }
    }

    fn setup_if_not_exists(path: &std::path::PathBuf) {
        if !path.exists() {
            std::fs::create_dir_all(path).unwrap();
        }
    }
}
