use burn::module::AutodiffModule;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use std::time::Instant;

use crate::backend::{Device, Gpu};
use crate::checkpoint;
use crate::data::get_batch;
use crate::model::{FoxAiModel, ModelConfig};

pub type TrainBackend = crate::backend::TrainGpu;

fn val_loss(
    model: &FoxAiModel<TrainBackend>,
    data: &[i32],
    config: &ModelConfig,
    device: &Device,
) -> f32 {
    if data.len() <= config.context_length + 1 {
        return 0.0;
    }

    let m = model.valid(); // no gradients needed
    let loss_fn = CrossEntropyLossConfig::new().init(device);

    let batches = 10;
    let mut total = 0.0f32;
    for _ in 0..batches {
        let (x, y) = get_batch::<Gpu>(data, 16, config.context_length, device);
        let logits = m.forward(x);
        let [b, t, v] = logits.dims();
        let loss = loss_fn.forward(logits.reshape([b * t, v]), y.reshape([b * t]));
        total += loss.into_scalar();
    }
    total / batches as f32
}

fn format_time(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{:02}h {:02}m {:02}s", h, m, s)
    } else {
        format!("{:02}m {:02}s", m, s)
    }
}

pub fn train(
    train_data: &[i32],
    val_data: &[i32],
    config: &ModelConfig,
    steps: usize, // how many training steps to run
) -> FoxAiModel<TrainBackend> {

    let device = crate::backend::device();

    println!("--------------------------------------------------");
    println!("🚀 FoxAi Training Engine");
    println!("Device:      {}", crate::backend::device_name());
    println!("Architecture: {} layers | {} d_model | {} heads | ctx {}", config.num_layers, config.d_model, config.num_heads, config.context_length);
    println!("Parameters:   ~{:.2}M trainable params", config.param_count() as f64 / 1e6);
    println!("Target Steps: {} | Warmup: 200", steps);
    println!("--------------------------------------------------");

    let mut model = FoxAiModel::<TrainBackend>::new(config, &device);

    let mut start = 0usize;
    if std::env::args().nth(2).as_deref() == Some("resume") {
        model = crate::checkpoint::load::<TrainBackend>(config, &device).expect("no checkpoint to resume from");
        start = std::fs::read_to_string("checkpoints/step.txt").ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        println!(">>> Resuming from step {}", start);
    }
    let mut optim = AdamConfig::new().init();
    let loss_fn = CrossEntropyLossConfig::new().with_pad_tokens(Some(vec![crate::tokenizer::PAD as usize])).init(&device);

    let batch_size = 32;
    let base_lr = 4e-4_f64;
    let min_lr = 4e-5_f64;
    let warmup = 200usize;
    let mut best_val = f32::INFINITY;
    let started = Instant::now();

    for step in start..=steps {
        // short warmup, then cosine decay to min_lr
        let lr = if step < warmup {
            base_lr * (step + 1) as f64 / warmup as f64
        } else {
            let progress = (step - warmup) as f64 / (steps - warmup).max(1) as f64;
            min_lr + 0.5 * (base_lr - min_lr) * (1.0 + (std::f64::consts::PI * progress).cos())
        };

        let lr = lr * ((step - start + 1) as f64 / 100.0).min(1.0);

        let (x, y) = crate::fim::get_fim_batch::<TrainBackend>(train_data, batch_size, config.context_length, 0.5, &device);

        let logits = model.forward(x);
        let [b, t, v] = logits.dims();
        let loss = loss_fn.forward(logits.reshape([b * t, v]), y.reshape([b * t]));

        if step % 50 == 0 || step == steps {
            let elapsed_secs = started.elapsed().as_secs_f64();
            let steps_done = (step - start + 1) as f64;
            let steps_remaining = steps.saturating_sub(step);
            let it_per_sec = steps_done / elapsed_secs.max(1e-3);
            let tok_per_sec = it_per_sec * (batch_size * config.context_length) as f64;
            let eta_secs = (steps_remaining as f64 / it_per_sec.max(1e-3)) as u64;
            let pct = (step as f64 / steps as f64) * 100.0;

            println!(
                "step {:>5}/{} [{:>5.1}%] | loss {:.4} | lr {:.2e} | {:>4.1} it/s ({:>5.1}k tok/s) | elapsed {} | ETA {}",
                step,
                steps,
                pct,
                loss.clone().into_scalar(),
                lr,
                it_per_sec,
                tok_per_sec / 1000.0,
                format_time(elapsed_secs as u64),
                format_time(eta_secs),
            );
        }

        if step > 0 && step % 250 == 0 {
            let val = val_loss(&model, val_data, config, &device);
            let is_best = val < best_val;
            if is_best {
                best_val = val;
            }
            println!("       >>> val loss {:.4}{}", val, if is_best { " [new best!]" } else { "" });
        }

        let grads = loss.backward();
        let grads = GradientsParams::from_grads(grads, &model);
        model = optim.step(lr, model, grads);

        if step > 0 && step % 500 == 0 {
            checkpoint::save(&model.clone().valid()).expect("checkpoint save failed");
            let _ = std::fs::write("checkpoints/step.txt", step.to_string());
            println!("\n>>> Checkpoint saved at step {}!", step);
        }
    }

    let total_duration = started.elapsed();
    println!(
        "\n🎉 Training complete in {} ({:.1} it/s avg)!",
        format_time(total_duration.as_secs()),
        (steps - start + 1) as f64 / total_duration.as_secs_f64().max(1e-3),
    );

    model
}