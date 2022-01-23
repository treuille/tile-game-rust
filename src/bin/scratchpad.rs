use tile_game::out_of_core::{calculate_hash, InMemoryIntSet, IntSet};

fn main() {
    // out_of_core::say_hello();

    // panic!("say_hello()");
    let int_set = InMemoryIntSet::new();
    for i in 0..4 {
        let contains_i = int_set.contains(i);

        let hash_i = calculate_hash(&i);

        println!("{i} -> {contains_i} ({hash_i})");
    }

    let adrien = "Adrien!!! ";
    println!("Hello {adrien}");
}
