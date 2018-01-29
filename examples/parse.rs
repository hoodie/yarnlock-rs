extern crate yarn_lock;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use yarn_lock::parse;

fn main() {
    if let Some(path) = env::args().nth(1) {
        let content = {
            let mut file = File::open(path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            content
        };
        
        println!("{:#?}", parse(&content))

    }
}

