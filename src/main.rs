use std::fs;
use std::io::{self, Write};
use std::process::Command;

fn main() {
    //automatically reads in current dir
    let paths = fs::read_dir("./").unwrap();
    for path in paths {
        let file = path.unwrap().path().display().to_string();
        let file_name = String::from(file);
        if file_name.ends_with(".in") {
            let output = Command::new("java").arg("Test.java").output().expect("run");
            println!("status: {}", output.status);
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        } else {
            println!("No input files");
            return;
        }
    }
}
