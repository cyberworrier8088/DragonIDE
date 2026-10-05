#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub vocab_size: usize,
    pub context_length: usize,
    pub d_model: usize,
    pub num_heads: usize,
    pub num_layers: usize,
}

impl ModelConfig {
    pub fn tiny(vocab_size: usize) -> Self {

        Self {
            vocab_size: vocab_size,
            context_length: 32,
            d_model: 64,
            num_heads: 4,
            num_layers: 2,
        }
    }
}