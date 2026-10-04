
use burn::backend::Wgpu;
use burn::tensor::{Int, Tensor};


pub fn test_batch(batch: &[Vec<usize>]) {
    let device = Default::default();

    // Convert Vec<Vec<usize>> into a fixed 2D array.
    let data: [[i64; 8]; 4] = [
        [
            batch[0][0] as i64,
            batch[0][1] as i64,
            batch[0][2] as i64,
            batch[0][3] as i64,
            batch[0][4] as i64,
            batch[0][5] as i64,
            batch[0][6] as i64,
            batch[0][7] as i64,
        ],
        [
            batch[1][0] as i64,
            batch[1][1] as i64,
            batch[1][2] as i64,
            batch[1][3] as i64,
            batch[1][4] as i64,
            batch[1][5] as i64,
            batch[1][6] as i64,
            batch[1][7] as i64,
        ],
        [
            batch[2][0] as i64,
            batch[2][1] as i64,
            batch[2][2] as i64,
            batch[2][3] as i64,
            batch[2][4] as i64,
            batch[2][5] as i64,
            batch[2][6] as i64,
            batch[2][7] as i64,
        ],
        [
            batch[3][0] as i64,
            batch[3][1] as i64,
            batch[3][2] as i64,
            batch[3][3] as i64,
            batch[3][4] as i64,
            batch[3][5] as i64,
            batch[3][6] as i64,
            batch[3][7] as i64,
        ],
    ];

    let tensor: Tensor<Wgpu, 2, Int> =
        Tensor::from_ints(data, &device);

    println!("Batch tensor:");
    println!("{}", tensor);
}