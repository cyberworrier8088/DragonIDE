
mod data;
mod generate;
mod model;
mod train;
mod checkpoint;
mod tokenizer;
mod corpus;
mod token_data;



use burn::backend::Wgpu;
use burn::module::AutodiffModule;
use model::ModelConfig;
use tokenizer::Tokenizer;


fn main() {

    let mode = std::env::args().nth(1).unwrap_or_else(|| "gen".to_string());

    match mode.as_str() {

        "corpus" => {
            corpus::build().expect("corpus failed");
        }
        "tok" => {
            let tok = tokenizer::train_and_save(corpus::CODE_FILE, tokenizer::VOCAB_SIZE, 8_000_000).expect("tokenizer failed");

            tokenizer::demo(&tok);
        }
        "data" => {
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            token_data::build(&tok, corpus::CODE_FILE, 40_000_000).expect("data failed");
        }
        "train" => {
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::small(tok.vocab_size());
            let (train_ids, val_ids) = token_data::load().expect("run `tok` first");
            println!("train tokens: {} | val tokens: {}", train_ids.len(), val_ids.len());

            let model = train::train(&train_ids, &val_ids, &config, 3000).valid();
            checkpoint::save(&model).expect("save failed");
            println!("saved to {}.mpk", checkpoint::MODEL_PATH);
        }

        _ => {
            let device = Default::default();
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::small(tok.vocab_size());
            let model = checkpoint::load::<Wgpu>(&config, &device).expect("no checkpoint, run `train` first");

            let text = generate::generate::<Wgpu>(
                &model,
                &tok,
                "fn main() {\n    let ",
                200,
                config.context_length,
                0.7,
                10,
                &device,
            );
            println!("{}", text);
        }
    }
}