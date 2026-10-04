use std::process::Command;
use std::fs;

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