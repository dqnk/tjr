use std::fs;
use std::io::{self, Write};
use std::process::Command;

fn main() {
    //automatically reads in current dir
    let paths = fs::read_dir("./").unwrap();
    let output = Command::new("java").arg("Test.java").output().expect("run");
    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display());
    }
}
