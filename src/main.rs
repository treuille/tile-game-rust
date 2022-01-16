use my_test_lib::do_someting;
use my_test_lib::front_of_house::do_something_else;

fn main() {
    println!("Hello, world!");
    do_someting_2();

    // Try to use the library functions
    do_someting();
    do_something_else();
}

fn do_someting_2() {
    println!("do_something_2");
}
