use burn::backend::{Autodiff, Wgpu};
use burn::module::AutodiffModule;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use std::time::Instant;

use crate::checkpoint;
use crate::data::get_batch;
use crate::model::{FoxAiModel, ModelConfig};

pub type TrainBackend = Autodiff<Wgpu>;

fn val_loss(model: &FoxAiModel<TrainBackend>, data: &[i32], config: &ModelConfig) -> f32 {

    let device = Default::default();
    let m = model.valid(); // no gradients needed
    let loss_fn = CrossEntropyLossConfig::new().init(&device);

    let batches = 10;
    let mut total = 0.0f32;
    for _ in 0..batches {
        let (x, y) = get_batch::<Wgpu>(data, 16, config.context_length, &device);
        let logits = m.forward(x);
        let [b, t, v] = logits.dims();
        let loss = loss_fn.forward(logits.reshape([b * t, v]), y.reshape([b * t]));
        total += loss.into_scalar();
    }
    total / batches as f32

}

pub fn train(
    train_data: &[i32],
    val_data: &[i32],
    config: &ModelConfig,
    steps: usize,
) -> FoxAiModel<TrainBackend> {

    let device = Default::default();

    let mut model = FoxAiModel::<TrainBackend>::new(config, &device);
    let mut optim = AdamConfig::new().init();
    let loss_fn = CrossEntropyLossConfig::new().init(&device);

    let batch_size = 32;
    let lr = 4e-4;
    let started = Instant::now();

    for step in 0..=steps {
        let (x, y) = get_batch::<TrainBackend>(train_data, batch_size, config.context_length, &device);

        let logits = model.forward(x);
        let [b, t, v] = logits.dims();
        let loss = loss_fn.forward(logits.reshape([b * t, v]), y.reshape([b * t]));

        if step % 50 == 0 {
            println!(
                "step {:>5} | loss {:.4} | {:.0}s",
                step,
                loss.clone().into_scalar(),
                started.elapsed().as_secs()
            );
        }

        if step % 250 == 0 {
            println!("       >>> val loss {:.4}", val_loss(&model, val_data, config));
        }

        let grads = loss.backward();
        let grads = GradientsParams::from_grads(grads, &model);
        model = optim.step(lr, model, grads);
        

        if step > 0 && step % 500 == 0 {
            checkpoint::save(&model.clone().valid()).expect("checkpoint save failed");
            println!("\n>>> checkpoint saved!");
        }
    }
    model
}