//! Demonstration of why 'static bounds are needed
//!
//! This file shows the exact compiler errors that occur when trying to use
//! types with non-static lifetimes in Arc<T>.

use std::sync::Arc;

fn main() {
    demonstrate_arc_static_requirement();
}

fn demonstrate_arc_static_requirement() {
    let temp_data = "temporary data".to_string();

    // This struct contains a non-static reference
    #[derive(Debug)]
    struct ContainsBorrowed<'a> {
        data: &'a str,
    }

    let borrowed_struct = ContainsBorrowed { data: &temp_data };

    // This line will FAIL to compile with error:
    // "borrowed_struct` does not live long enough"
    // "argument requires that `borrowed_struct` is borrowed for `'static`"
    let arc_borrowed = Arc::new(borrowed_struct);

    // If we could somehow create the Arc, this would be use-after-free:
    std::thread::spawn(move || {
        println!("Data: {:?}", arc_borrowed);
        // temp_data would be out of scope here!
    });
}

// This demonstrates why Arc<T> requires T: 'static
#[allow(dead_code)]
fn show_working_example() {
    // This works because String is owned data ('static)
    #[derive(Debug)]
    struct ContainsOwned {
        data: String,
    }

    let owned_struct = ContainsOwned {
        data: "owned data".to_string(),
    };
    let arc_owned = Arc::new(owned_struct); // ✅ This works!

    std::thread::spawn(move || {
        println!("Data: {:?}", arc_owned); // ✅ Safe!
    });
}
