extern crate yarn_lock;

use std::env;

fn main() {
    if let Some(path) = env::args().nth(1) {
        match yarn_lock::open_by_name(&path) {
            Ok(locks) => for (name, locks) in &locks {
                println!("{:#}", name);
                for lock in locks {
                    let version = lock.version
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(String::new);
                    let last_seen = lock.last_seen
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(String::new);
                    println!(" * {:} -> {}", last_seen, version)
                }
            },
            Err(e) => println!("{:?}", e),
        }
    }
}
