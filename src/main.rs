use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

fn find_main_java(path: &Path) {
    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let file = path.unwrap().path().display().to_string();
        let file_name = String::from(file);
        println!("{}", file_name);

        if file_name.ends_with(".java") {
            file_name;
        }
    }
}

fn main() {
    //automatically reads in current dir
    let args: Vec<String> = env::args().collect();
    let test_dir: &Path;
    let program_name: String;
    match args.len() {
        1 => {
            test_dir = Path::new(".");
            //fix
            program_name = find_main_java();
        }
        2 => test_dir = Path::new("."),
        _ => println!("Error"),
    }
    if args.len() != 3 {
        println!("wrong input");
        return;
    }

    let program_name = &args[1];
    let test_dir = Path::new(&args[2]);

    println!(
        "test dir: {}, program name: {}",
        test_dir.display().to_string(),
        program_name
    );

    let paths = fs::read_dir(test_dir).unwrap();

    for path in paths {
        let file = path.unwrap().path().display().to_string();
        let file_name = String::from(file);
        println!("{}", file_name);

        if file_name.ends_with(".in") {
            let output = Command::new("java").arg("Test.java").output().expect("run");
            println!("status: {}", output.status);
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
    }
}
