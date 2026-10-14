
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
mod benchmarks;
mod fimdata;




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
            let config = ModelConfig::current(tok.vocab_size());
            let (rows, n_train, n_total) = fimdata::load().expect("run 'fimdata' first");
            train::train(&rows, n_train, n_total, &config);
        }
        "fimdata" => {
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run 'tok' first");
            let n: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(300_000);
            fimdata::build(&tok, corpus::CODE_FILE, n).expect("fimdata failed");
        }
        "fim" => {
            let device = backend::device();
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::current(tok.vocab_size());
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
            let config = ModelConfig::current(tok.vocab_size());

            let args: Vec<String> = std::env::args().collect();
            let model_path = if let Some(pos) = args.iter().position(|a| a == "-path" || a == "--path") {
                args.get(pos + 1).map(|s| s.as_str()).unwrap_or(checkpoint::MODEL_PATH)
            } else {
                args.get(2).map(|s| s.as_str()).unwrap_or(checkpoint::MODEL_PATH)
            };

            let clean_path = checkpoint::resolve_path(model_path);
            let file_path = format!("{}.mpk", clean_path);
            let file_size_mb = std::fs::metadata(&file_path)
                .or_else(|_| std::fs::metadata(&clean_path))
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

            let model = checkpoint::load_from::<backend::Gpu>(&clean_path, &config, &device)
                .expect("Failed to load model. Check path or run `train` first");

            let prompts = benchmarks::get_prompts();

            for (i, (name, prefix, suffix)) in prompts.iter().enumerate() {
                println!("--- [Test {}/{}] {} ---", i + 1, prompts.len(), name);
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

        "fimeval" => {
            let device = backend::device();
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run 'tok' first");
            let config = ModelConfig::current(tok.vocab_size());
            let path = std::env::args().nth(2).unwrap_or_else(|| checkpoint::MODEL_PATH.to_string());
            let model = checkpoint::load_from::<backend::Gpu>(&path, &config, &device).expect("load failed");
            let (rows, n_train, n_total) = fimdata::load().expect("run 'fimdata' first");


            let (mut exact, mut total) = (0, 0);
            for r in n_train..n_total.min(n_train + 60) {
                let row = &rows[r * fimdata::ROW..(r + 1) * fimdata::ROW];
                let Some(p) = row.iter().position(|&t| t as u32 == tokenizer::FIM_MIDDLE) else { continue; };
                let prompt: Vec<i32> = row[..=p].iter().map(|&t| t as i32).collect();
                let truth: Vec<u32> = row[p + 1..].iter().map(|&t| t as u32).take_while(|&t| t != tokenizer::PAD && t != tokenizer::EOS).collect();
                let out = generate::generate_ids::<backend::Gpu>(&model, prompt, 60, config.context_length, 0.05, 1, &device);
                let got: Vec<u32> = out.iter().map(|&t| t as u32).take_while(|&t| t != tokenizer::EOS).collect();
                total += 1;

                if got == truth { exact += 1; }
                if total <= 8 {
                    println!("TRUTH: {:?}\nGOT:   {:?}\n", tok.decode(&truth), tok.decode(&got));
                }
            }

            println!("exact match: {}/{}", exact, total);
        }

        _ => {
            let device = backend::device();
            let tok = Tokenizer::load(tokenizer::TOKENIZER_FILE).expect("run `tok` first");
            let config = ModelConfig::current(tok.vocab_size());
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