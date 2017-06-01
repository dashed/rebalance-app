extern crate csv;

use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Read;


fn main() {

    let contents = read_file_to_string("assets/fundaccountdetails.csv");

    println!("Hello, world!");
}

fn read_file_to_string<T: AsRef<Path>>(path_to_file: T) -> String {

    let path = Path::new(path_to_file.as_ref());
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => contents
    }

}
