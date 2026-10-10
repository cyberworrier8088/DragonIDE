#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub vocab_size: usize,
    pub context_length: usize,
    pub d_model: usize,
    pub num_heads: usize,
    pub num_layers: usize,
}

#[allow(dead_code)]
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

    pub fn small(vocab_size: usize) -> Self {
        Self {
            vocab_size,
            context_length: 128,
            d_model: 128,
            num_heads: 4,
            num_layers: 4,
        }
    }
}