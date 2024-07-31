use core::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Gama(pub f32);

impl Gama {
    pub const CHUNK_TYPE: &'static str = "gAMA";

    pub fn parse(content: &[u8]) -> Self {
        assert!(content.len() == 4);
        let value = u32::from_be_bytes(content.try_into().unwrap()) as f32;
        Self(value / 100000.0)
    }
}

impl fmt::Display for Gama {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gama: {}", self.0)
    }
}
