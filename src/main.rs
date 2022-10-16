use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

fn find_main_java<'a>(pat: Box<&Path>) -> Box<&'a Path> {
    let paths = fs::read_dir(*pat).unwrap();
    for path in paths {
        //        let file2 = Path::new(&path.unwrap().path().display().to_string());
        //       let file = file2.clone().extension().and_then(OsStr::to_str);
        let pat2 = Box::new(Path::new(&path.unwrap().path().display().to_string()));
        return pat2;
    }
    panic!("error finding main.java file");
}

fn main() {
    //automatically reads in current dir
    let args: Vec<String> = env::args().collect();
    let test_dir: &Path;
    let program_name: &Path;
    match args.len() {
        1 => {
            test_dir = Path::new(".");
            let test_box = Box::new(test_dir);
            program_name = find_main_java(test_box);
        }
        2 => {
            if args[1].ends_with(".java") {
                test_dir = Path::new(".");
                program_name = Path::new(&args[1]);
            } else {
                test_dir = Path::new(&args[1]);
                program_name = find_main_java(&test_dir);
            }
        }
        _ => {
            test_dir = Path::new("a");
            program_name = find_main_java(&test_dir);
            println!("Error");
        }
    }

    println!(
        "{}, {}",
        String::from(test_dir.display().to_string()),
        String::from(program_name.display().to_string()),
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
