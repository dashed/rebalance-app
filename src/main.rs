extern crate csv;

use std::path::Path;
use std::fs::File;
use std::error::Error;


fn main() {

    let path = Path::new("assets/fundaccountdetails.csv");

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => {
            let display = path.display();
panic!("couldn't open {}: {}", display,
                                                   why.description())
        },
        Ok(file) => file,
    };

    println!("Hello, world!");
}
