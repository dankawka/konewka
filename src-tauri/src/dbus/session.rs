pub struct Session {
    pub path: String,
}

impl Session {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn forward_logs(&self) {
        println!("Forwarding logs for session: {}", self.path);
    }
}
