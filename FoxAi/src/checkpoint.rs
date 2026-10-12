use burn::module::Module;
use burn::prelude::*;
use burn::record::CompactRecorder;

use crate::model::{FoxAiModel, ModelConfig};

pub const CHECKPOINT_DIR: &str = "checkpoints";
pub const MODEL_PATH: &str = "checkpoints/foxai";

pub fn save<B: Backend>(model: &FoxAiModel<B>) -> Result<(), String> {

    std::fs::create_dir_all(CHECKPOINT_DIR).map_err(|e| e.to_string())?;

    model.clone().save_file(MODEL_PATH, &CompactRecorder::new()).map_err(|e| format!("Could not save model: {}", e))
}

pub fn resolve_path(input: &str) -> String {
    let raw = input.trim();
    let candidates = [
        raw.to_string(),
        raw.strip_prefix("FoxAi/").unwrap_or(raw).to_string(),
        raw.strip_prefix("FoxAi\\").unwrap_or(raw).to_string(),
        format!("FoxAi/{}", raw),
    ];

    for c in &candidates {
        let clean = c.strip_suffix(".mpk").unwrap_or(c);
        let with_mpk = format!("{}.mpk", clean);
        if std::path::Path::new(&with_mpk).exists() || std::path::Path::new(clean).exists() {
            return clean.to_string();
        }
    }

    raw.strip_suffix(".mpk").unwrap_or(raw).to_string()
}

pub fn load<B: Backend>(
    config: &ModelConfig,
    device: &B::Device,
) -> Result<FoxAiModel<B>, String> {
    load_from(MODEL_PATH, config, device)
}

pub fn load_from<B: Backend>(
    path: &str,
    config: &ModelConfig,
    device: &B::Device,
) -> Result<FoxAiModel<B>, String> {
    let clean = resolve_path(path);
    FoxAiModel::<B>::new(config, device)
        .load_file(&clean, &CompactRecorder::new(), device)
        .map_err(|e| format!("Could not load model from '{}': {}", clean, e))
}