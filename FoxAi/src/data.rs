use std::process::Command;
use std::fs;
use std::collections::HashMap;
use rand::RngExt;

pub fn get_data_set() {
    let url = "https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt";
    let output_dir = "dataset";
    let output_file = format!("{}/test-data.txt", output_dir);

    println!("Creating folder '{}'..", output_dir);


    if let Err(e) = fs::create_dir_all(output_dir) {
        println!("Failed men creating dir: {}", e);
        return;
    } 



    println!("Downloading...");

    // run the native 'curl' command built into your OS
    let status = Command::new("curl")
    .arg("-L")
    .arg("-o")
    .arg(&output_file)
    .arg(url)
    .status()
    .expect("Bro sorry. i am...");


    if status.success() {
        println!("Downloaded Your Wife!");
    } else {
        println!("downlaod Failed");
    }
}


pub fn data_set_preparing() {

    let text = fs::read_to_string("dataset/test-data.txt").unwrap();


    // Craete a vocabularys
    let mut chars: Vec<char> = text.chars().collect();

    chars.sort();
    chars.dedup();

    let vocab_size = chars.len();

    println!("{}", chars.iter().collect::<String>());
    println!("{}", vocab_size);


    // character -> integer
    let mut stoi = HashMap::new();

    // integer -> character
    let mut itos = HashMap::new();


    for (i, &ch) in chars.iter().enumerate() {
        stoi.insert(ch, i);
        itos.insert(i, ch);
    }

    // Encodee the entir dataset
    let mut data = Vec::new();

    for ch in text.chars() {
        data.push(stoi[&ch]);
    }

    println!("Number of Tokens: {}", data.len());

    // first 1000 tokens
    println!("{:?}", &data[..1000]);

    let n = (0.9 * data.len() as f64) as usize;

    let train_data = &data[..n];
    let val_data = &data[n..];

    println!("Train: {}", train_data.len());
    println!("Validation: {}", val_data.len());

    let (xb, yb) = get_batch(train_data, 4, 8);

    println!("Input batch:");
    println!("{:?}", xb);

    println!("Target batch:");
    println!("{:?}", yb);
}

fn get_batch(
    data: &[usize],
    batch_size: usize,
    block_size: usize,
) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {

    let mut rng = rand::rng();

    let mut x = Vec::new();
    let mut y = Vec::new();

    for _ in 0..batch_size {
        let i = rng.random_range(0..data.len() - block_size);

        let input = data[i..i + block_size].to_vec();
        let target = data[i + 1..i + block_size + 1].to_vec();

        x.push(input);
        y.push(target);
    }

    (x, y)
}