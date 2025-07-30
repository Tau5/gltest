
pub struct OpenxrProps {
    pub ipd_scale: f32,
    pub roomscale_scale: f32
}

impl Default for OpenxrProps {
    fn default() -> Self {
        Self {
            ipd_scale: 1.0f32,
            roomscale_scale: 1.0f32
        }
    }
}