use std::{sync::Arc, thread};



struct Pane {
    test: String
}



fn test() {
    let a = Arc::new(Pane {
        test: String::from("hello world")
    });

    let b = Arc::clone(&a);
    thread::spawn(move || {
        println!("{}", b.test);
    });
}


