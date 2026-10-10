use burn::backend::{Autodiff, Wgpu};
use burn::nn::loss::CrossEntropyLossConfig;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};


use crate::data::{get_batch, CharDataset};
use crate::model::{FoxAiModel, ModelConfig};

pub type TrainBackend = Autodiff<Wgpu>;

pub fn train(ds: &CharDataset, config: &ModelConfig) -> FoxAiModel<TrainBackend>{

    let device = Default::default();

    let mut model = FoxAiModel::<TrainBackend>::new(config, &device);
    let mut optim = AdamConfig::new().init();
    let loss_fn = CrossEntropyLossConfig::new().init(&device);
    
    let steps = 5000;
    let batch_size = 32;
    let lr = 1e-3;

    for step in 0..=steps {

        let (x, y) = get_batch::<TrainBackend>(
            &ds.train,
            batch_size,
            config.context_length,
            &device,
        );

        let logits = model.forward(x);
        let [b, t, v] = logits.dims();

        // flatten: [B, T, V] -> [B*T, V] and targets [B, T] -> [B*T]
        let loss = loss_fn.forward(logits.reshape([b * t, v]), y.reshape([b* t]));

        if step % 50 == 0 {
            println!("Step {:>4} | loss {}", step, loss.clone().into_scalar());
        }

        let grads = loss.backward();
        let grads = GradientsParams::from_grads(grads, &model);
        model = optim.step(lr, model, grads);
    }

    model
}