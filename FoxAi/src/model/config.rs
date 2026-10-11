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
            context_length: 256,
            d_model: 192,
            num_heads: 6,
            num_layers: 6,
        }
    }

    pub fn medium(vocab_size: usize) -> Self {
        Self {
            vocab_size,
            context_length: 512,
            d_model: 256,
            num_heads: 8,
            num_layers: 8,
        }
    }

    pub fn large(vocab_size: usize) -> Self {
        Self {
            vocab_size,
            context_length: 512,
            d_model: 384,
            num_heads: 12,
            num_layers: 12,
        }
    }

    /// Calculate approximate total trainable parameters
    pub fn param_count(&self) -> usize {
        let embeddings = (self.vocab_size + self.context_length) * self.d_model;
        let per_block_attn = 4 * (self.d_model * self.d_model + self.d_model);
        let per_block_ffn = 2 * (self.d_model * (self.d_model * 4) + (self.d_model * 4));
        let per_block_norm = 4 * self.d_model;
        let blocks = self.num_layers * (per_block_attn + per_block_ffn + per_block_norm);
        let final_norm = 2 * self.d_model;
        let lm_head = self.d_model * self.vocab_size + self.vocab_size;
        embeddings + blocks + final_norm + lm_head
    }
}