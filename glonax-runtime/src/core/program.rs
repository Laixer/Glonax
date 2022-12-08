#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProgramArgument {
    /// Program ID.
    pub id: u32,
    /// Program parameters.
    pub parameters: Vec<f32>,
}
