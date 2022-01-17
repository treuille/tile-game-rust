#![allow(unused)]
use my_test_lib::do_someting;

// #[allow(unused_imports)]
use my_test_lib::front_of_house::do_something_else;

fn main() {
    println!("Hello, world!");
    chapter_3();
    // do_someting_2();

    // // Try to use the library functions
    // do_someting();
    // do_something_else();

    // // // Now let's test this panic thing
    // // panic!("Crash and burn.");
}

const SOMETHING_PRIME: i32 = 1 + 10;

fn chapter_3() {
    let mut x = 5;
    println!("x: {}", x);
    x = 7;
    println!("x: {}", x);
    println!("SOMETHING_PRIME: {}", SOMETHING_PRIME);

    // match "-42".parse::<i32>() {
    //     Ok(guess) => println!("guess: {}", guess),
    //     Err(err) => println!("not a number: {}", err),
    // }
    if let Ok(guess) = "-42".parse::<i32>() {
        println!("guess: {}", guess);
    }
}

// #[allow(dead_code)]
fn do_someting_2() {
    println!("do_something_2");
}
