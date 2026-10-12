
mod data;
mod generate;
mod model;
mod train;
mod checkpoint;
mod tokenizer;
mod corpus;
mod token_data;
mod fim;
mod backend;




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
            token_data::build(&tok, corpus::CODE_FILE, 150_000_000).expect("data failed");
        }
        "train" => {
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::medium(tok.vocab_size());
            let (train_ids, val_ids) = token_data::load().expect("run `tok` first");
            println!("train tokens: {} | val tokens: {}", train_ids.len(), val_ids.len());

            let model = train::train(&train_ids, &val_ids, &config, 30000).valid();
            checkpoint::save(&model).expect("save failed");
            println!("saved to {}.mpk", checkpoint::MODEL_PATH);
        }
        "fim" => {
            let device = backend::device();
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::medium(tok.vocab_size());
            let model = checkpoint::load::<backend::Gpu>(&config, &device).expect("no checkpoint, run `train` first");
            
            let out = generate::complete::<backend::Gpu>(
                &model,
                &tok,
                "fn add(a: i32, b: i32) ->  i32 {\n    ",
                "\n}",
                40,
                config.context_length,
                0.4,
                5,
                &device,
            );
            println!("[{}]", out);
        }

        "fimtest" => {
            let device = backend::device();
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::medium(tok.vocab_size());

            let args: Vec<String> = std::env::args().collect();
            let model_path = if let Some(pos) = args.iter().position(|a| a == "-path" || a == "--path") {
                args.get(pos + 1).map(|s| s.as_str()).unwrap_or(checkpoint::MODEL_PATH)
            } else {
                args.get(2).map(|s| s.as_str()).unwrap_or(checkpoint::MODEL_PATH)
            };

            let clean_path = model_path.strip_suffix(".mpk").unwrap_or(model_path);
            let file_path = format!("{}.mpk", clean_path);
            let file_size_mb = std::fs::metadata(&file_path)
                .or_else(|_| std::fs::metadata(model_path))
                .map(|m| m.len() as f64 / 1_048_576.0)
                .unwrap_or(0.0);

            println!("==================================================");
            println!("🦊 FoxAi FIM Benchmark & Model Inspection");
            println!("Model Path:   {}", file_path);
            println!("File Size:    {:.2} MB", file_size_mb);
            println!("Config:       {} layers | {} d_model | {} heads | ctx {}", config.num_layers, config.d_model, config.num_heads, config.context_length);
            println!("Parameters:   ~{:.2}M params", config.param_count() as f64 / 1e6);
            println!("Device:       {}", backend::device_name());
            println!("==================================================\n");

            let model = checkpoint::load_from::<backend::Gpu>(clean_path, &config, &device)
                .expect("Failed to load model. Check path or run `train` first");

            let prompts = [
                ("Function Body (Addition)", "fn add(a: i32, b: i32) -> i32 {\n    ", "\n}"),
                ("Struct Definition", "pub struct Point {\n    pub x: f32,\n    ", "\n}"),
                ("Result Pattern Match", "match fs::read_to_string(path) {\n    Ok(content) => ", ",\n    Err(e) => return Err(e.to_string()),\n}"),
                ("Iterator Mapping", "let numbers = vec![1, 2, 3, 4, 5];\nlet doubled: Vec<i32> = numbers.iter().map(|", ").collect();"),
                ("Constructor Implementation", "impl Config {\n    pub fn new(name: &str) -> Self {\n        Self {\n            name: ", ",\n        }\n    }\n}"),
                ("Boolean Condition Check", "fn is_even(n: i32) -> bool {\n    ", "\n}"),
                ("String Formatting", "let msg = format!(\"User: {}, Status: {}\", ", ");"),
                ("Vector Filter Condition", "for item in &items {\n    if item.", " {\n        continue;\n    }\n}"),
                ("Getter Method", "pub fn get_id(&self) -> u64 {\n    ", "\n}"),
                ("File Write Helper", "pub fn save_file(path: &str, data: &[u8]) -> std::io::Result<()> {\n    ", "\n}"),
            ];

            for (i, (name, prefix, suffix)) in prompts.iter().enumerate() {
                println!("--- [Test {}/10] {} ---", i + 1, name);
                println!("Prefix:\n{}", prefix);
                println!("Suffix:\n{}", suffix);
                let out = generate::complete::<backend::Gpu>(
                    &model,
                    &tok,
                    prefix,
                    suffix,
                    40,
                    config.context_length,
                    0.3,
                    5,
                    &device,
                );
                println!("Completed:\n{}\n", out);
            }
        }

        _ => {
            let device = backend::device();
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::medium(tok.vocab_size());
            let model = checkpoint::load::<backend::Gpu>(&config, &device).expect("no checkpoint, run `train` first");

            let text = generate::generate::<backend::Gpu>(
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