use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn find_main_java(path: &Path, mut java_path: PathBuf) -> PathBuf {
    let paths = fs::read_dir(path).unwrap();
    for pat2 in paths {
        let patcopy = String::from(pat2.unwrap().path().display().to_string());
        if patcopy.ends_with(".java") {
            java_path.push(patcopy);
            return java_path;
        }
    }
    panic!("error finding main.java file");
}

fn main() {
    //automatically reads in current dir
    let args: Vec<String> = env::args().collect();
    let test_dir: &Path;
    let mut program_name: &Path = Path::new(".");
    let mut program_name_string: PathBuf = PathBuf::new();
    match args.len() {
        1 => {
            test_dir = Path::new(".");
            program_name_string = find_main_java(test_dir, program_name_string);
        }
        2 => {
            if args[1].ends_with(".java") {
                test_dir = Path::new(".");
                program_name = Path::new(&args[1]);
            } else {
                test_dir = Path::new(&args[1]);
                program_name_string = find_main_java(test_dir, program_name_string);
            }
        }
        _ => {
            test_dir = Path::new("a");
            program_name_string = find_main_java(test_dir, program_name_string);
            println!("Error");
        }
    }

    println!(
        "{}, {}",
        String::from(test_dir.display().to_string()),
        String::from(program_name_string.display().to_string()),
    );
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
    return;

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
