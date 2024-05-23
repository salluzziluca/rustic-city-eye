#[derive(Debug)]
pub struct Camera {
    //camera_client: Client,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Camera {
        Self {}
    }
}
