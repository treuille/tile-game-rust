use rayon::prelude::*;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Hello, world.");

    let numbers: Vec<_> = (0..20).collect();
    numbers.par_iter().for_each(|i| {
        println!("{i}");
        let one_second = Duration::from_secs(1);
        thread::sleep(one_second);
    });
}
