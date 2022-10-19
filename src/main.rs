use std::env;
use async_std;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
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

#[async_std::main]
async fn main() {
    // read args provided to command from CLI
    let args: Vec<String> = env::args().collect();
    let test_dir: &Path;
    let program_name: PathBuf;
    let mut children = vec![];

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
    //contains all files, not just tests, but will be filtered later in for loop
    let tests = fs::read_dir(test_dir).unwrap().into_iter();

    for test in tests {
        let program_name = program_name.clone();
        let file = PathBuf::from(test.unwrap().path());
        if file.extension() == None {
            continue;
            //TODO need serious cleanup here
        } else {
            // TODO pipe the input file
            if file.extension().unwrap() == "in" {
                children.push(async_std::task::spawn({
                    let program_name = program_name.clone();
                    let file = file.clone();
                    async move {
                    let file_stem = file.file_stem().unwrap().to_str().unwrap();
                    let output = Command::new("java")
                        .arg(&program_name)
                        .stdin(File::open(format!("{}.in", file_stem)).unwrap())
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
                        fs::write(format!("{}.res", file_stem), &output.stdout)
                            .expect("Unable to write output file");
                        //diff should return nothing
                        let out_file = PathBuf::from(format!("{}.out", file_stem));
                        let res_file = PathBuf::from(format!("{}.res", file_stem));
                        let output_diff = Command::new(format!("diff"))
                            .arg(&out_file)
                            .arg(&res_file)
                            .output()
                            .expect("run");
                        let out = String::from_utf8(output_diff.stdout).unwrap();
                        if out == "" {
                            println!("fine");
                        } else {
                            println!("not fine {}", out);
                        }
                    }
                }}));
            }
        }
    }
    for child in children {
        let _ = child.await;
    }
}
