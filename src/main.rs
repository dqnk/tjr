#![feature(let_else)]
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn find_main_java(path: &Path) -> PathBuf {
    let paths = fs::read_dir(path).unwrap();
    for p in paths {
        if p.as_ref()
            .unwrap()
            .path()
            .as_path()
            .extension()
            .and_then(OsStr::to_str)
            == Some("java")
        {
            return p.unwrap().path();
        }
    }
    panic!("error finding main.java file");
}

fn main() {
    // read args provided to command from CLI
    let args: Vec<String> = env::args().collect();
    let test_dir: &Path;
    let program_name: PathBuf;

    // obtain test_dir and program_name depending on the args provided
    // args.len() returns number of args including the executable name stored in &args[0]
    match args.len() {
        1 => {
            //assume everything is happening in current dir
            test_dir = Path::new(".");
            program_name = find_main_java(test_dir);
        }
        2 => {
            // java program is provided, tests are in current dir
            if args[1].ends_with(".java") {
                test_dir = Path::new(".");
                program_name = PathBuf::from(&args[1]);
            } else {
                // test dir is provided, java program is in current dir
                test_dir = Path::new(&args[1]);
                program_name = find_main_java(Path::new("."));
            }
        }
        3 => {
            // java program and test dir are provided, order does not matter
            // should program name be exected without ".java" filetype
            if args[1].ends_with(".java") {
                program_name = PathBuf::from(&args[1]);
                test_dir = Path::new(&args[2]);
            } else {
                program_name = PathBuf::from(format!("{}.java", &args[1]));
                test_dir = Path::new(&args[2]);
            }
        }
        _ => {
            panic!("too many/too few arguments")
        }
    }
    let mut i = 0;
    //contains all files, not just tests, but will be filtered later in for loop
    let tests = fs::read_dir(test_dir).unwrap().into_iter();

    for test in tests {
        let file = test.unwrap().path();
        if file.extension() == None {
            continue;
            //TODO need serious cleanup here
        } else {
            if file.extension().unwrap() == "in" {
                let output = Command::new("java")
                    .arg(&program_name)
                    .output()
                    .expect("run");
                // TODO: write output to .res file

                // panics should probably just be a println
                if output.status.code().unwrap() < 0 {
                    io::stderr().write_all(&output.stderr).unwrap();
                    panic!("Something is wrong with your OS: error {}", output.status);
                } else if output.status.code().unwrap() > 0 {
                    io::stderr().write_all(&output.stderr).unwrap();
                    panic!("Error compiling/running: {}", output.status);
                } else {
                    fs::write(
                        format!("{}.out", file.file_stem().unwrap().to_str().unwrap()),
                        &output.stdout,
                    )
                    .expect("Unable to write output file");
                }
            }
        }
        println!("{}", i);
        i += 1;
    }
    println!("here");
}
