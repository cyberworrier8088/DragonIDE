use burn::module::AutodiffModule;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use std::time::Instant;

use crate::backend::{Device, Gpu};
use crate::fimdata;
use crate::checkpoint;
use crate::model::{FoxAiModel, ModelConfig};
use crate::tokenizer::PAD;


pub type TrainBackend = crate::backend::TrainGpu;

const STATE_FILE: &str = "checkpoints/state.txt";

fn arg_val(name: &str) -> Option<String> {
    let prefix = format!("--{}=", name);
    for a in std::env::args() {
        if let Some(v) = a.strip_prefix(prefix.as_str()) {
            return Some(v.to_string());
        }
    }
    None
}

fn format_time(seconds: u64) -> String {
    format!("{:02}h {:02}m {:02}s", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
}

fn val_loss(model: &FoxAiModel<TrainBackend>, rows: &[u16], lo: usize, hi: usize, device: &Device) -> f32 {
    let m = model.valid();
    let loss_fn = CrossEntropyLossConfig::new().with_pad_tokens(Some(vec![PAD as usize])).init(device);
    let batches = 20;
    let mut total = 0.0f32;

    for _ in 0..batches {
        let (x, y) = fimdata::batch::<Gpu>(rows, lo, hi, 16, true, device);
        let logits = m.forward(x);
        let [b, t, v] = logits.dims();
        let loss = loss_fn.forward(logits.reshape([b * t, v]), y.reshape([b * t]));
        total += loss.into_scalar();
    }

    total / batches as f32
}

// useage: train [resume] [--hours=10] [--batch=32]]
pub fn train(rows: &[u16], n_train: usize, n_total: usize, config: &ModelConfig) {

    let device = crate::backend::device();
    assert_eq!(config.context_length, fimdata::ROW - 1, "context_length must be 512 for fim.bin");

    let hours: f64 = arg_val("hours").and_then(|s| s.parse().ok()).unwrap_or(10.0);
    let batch_size: usize = arg_val("batch").and_then(|s| s.parse().ok()).unwrap_or(32);
    let budget = hours * 3600.0;
    let resume = std::env::args().any(|a| a == "resume");

    println!("----------------------------------");
    println!("FoxAi Training engineee");
    println!("Device:      {}", crate::backend::device_name());
    println!("Architeture: {} layers | {} d_model | {} heads | cts {}", config.num_layers, config.d_model, config.num_heads, config.context_length);
    println!("Parameters: ~{:.2}M", config.param_count() as f64 / 1e6);
    println!("Time budget: {:.1} h | batch {} | train rows {}", hours, batch_size, n_train);
    println!("-----------------------------------");

    let mut model = FoxAiModel::<TrainBackend>::new(config, &device);
    let mut step = 0usize;
    let mut base_elapsed = 0.0f64;
    let mut best_val = f32::INFINITY;

    if resume {
        model = checkpoint::load::<TrainBackend>(config, &device).expect("no checkpoint to resume from");
        let s = std::fs::read_to_string(STATE_FILE).expect("no state.txt to resume from");
        let p: Vec<&str> = s.split_whitespace().collect();
        step = p[0].parse().unwrap();
        base_elapsed = p[1].parse().unwrap();
        best_val = p[2].parse().unwrap_or(f32::INFINITY);
        println!(">>> Resuming from step {} ({} already used)", step, format_time(base_elapsed as u64));
    }

    let start = step;

    let mut optim = AdamConfig::new().init();
    let loss_fn = CrossEntropyLossConfig::new().with_pad_tokens(Some(vec![PAD as usize])).init(&device);

    let base_lr = 3e-4_f64;
    let min_lr = 3e-5_f64;
    let started = Instant::now();

    let save_state = |step: usize, elapsed: f64, best: f32| {
        let _ = std::fs::write(STATE_FILE, format!("{} {} {}", step, elapsed, best));
    };

    loop {
        let elapsed = base_elapsed + started.elapsed().as_secs_f64();
        let progress = (elapsed / budget).min(1.0);

        if progress >= 1.0 {
            break;
        }

        // warmup, cosine decay over the TIME budget, short re-warmup after a resume
        let mut lr = min_lr + 0.5 * (base_lr - min_lr) * (1.0 + (std::f64::consts::PI * progress).cos());
        lr *= ((step + 1) as f64 / 200.0).min(1.0);
        lr *= ((step - start + 1) as f64 / 100.0).min(1.0);

        let (x, y) = fimdata::batch::<TrainBackend>(rows, 0, n_train, batch_size, false, &device);
        let logits = model.forward(x);
        let [b, t, v] = logits.dims();
        let loss = loss_fn.forward(logits.reshape([b * t, v]), y.reshape([b * t]));

        if step % 50 == 0 {
            let it_s = (step - start + 1) as f64 / started.elapsed().as_secs_f64().max(1e-3);

            println!("step {:>6} | loss {:.4} | lr {:.2e} | {:.2} it/s | used {} | left {}",
            step, loss.clone().into_scalar(), lr, it_s,
            format_time(elapsed as u64),
            format_time((budget - elapsed).max(0.0) as u64)
        );
        }


        let grads = loss.backward();
        let grads = GradientsParams::from_grads(grads, &model);
        model = optim.step(lr, model, grads);
        step += 1;

        if step % 500 == 0 {
            let elapsed = base_elapsed + started.elapsed().as_secs_f64();
            let val = val_loss(&model, rows, n_train, n_total, &device);
            let is_best = val < best_val;

            if is_best {
                best_val = val;
            }

            println!("       >>> FIM val loss {:.4}{}", val, if is_best { " [new best!]"} else { "" });

            let m = model.clone().valid();
            checkpoint::save(&m).expect("save failed");

            if is_best {
                checkpoint::save_to(&m, "checkpoints/best").expect("save best failed");
            }

            if step % 2000 == 0 {
                checkpoint::save_to(&m, &format!("checkpoints/history/foxai_step{}", step)).expect("save history failed");
            }

            save_state(step, elapsed, best_val);
            println!(">>> saved at step {}", step);
        }
    }


    // time is up: final model
    let elapsed = base_elapsed + started.elapsed().as_secs_f64();
    let m = model.valid();
    checkpoint::save(&m).expect("save failed");
    checkpoint::save_to(&m, "checkpoints/final").expect("save final failed");
    save_state(step, elapsed, best_val);
    println!("\nTraining finished: {} steps in {}. Saved checkpoints/final.mpk", step, format_time(elapsed as u64));

}