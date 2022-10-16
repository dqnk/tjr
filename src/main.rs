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
                test_dir = Path::new(&args[2]);
                program_name = PathBuf::from(&args[1]);
            } else {
                test_dir = Path::new(&args[1]);
                program_name = PathBuf::from(&args[2]);
            }
        }
        _ => {
            panic!("too many/too few arguments")
        }
    }

    println!(
        "{}, {}",
        String::from(test_dir.display().to_string()),
        String::from(program_name.display().to_string()),
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
