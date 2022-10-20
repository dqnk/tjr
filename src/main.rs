use std::env;
use async_std;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn find_main_java(path: &Path) -> Result<PathBuf, io::Error> {
    let paths = fs::read_dir(path).expect("Could not find target test dir").into_iter();
    for p in paths {
        let file = match p {
            Ok(file) => file.path(),
            Err(error) => return Err(error),
        };
        match file.extension(){
            Some(extension) => {
                if extension == "java" {
                    return Ok(file);
                }
            },
            None => {},
        }
    }
    panic!("error finding main.java file");
}

async fn thread(program_name: PathBuf, file: &Path) {
    // how?
    let file_stem = file.file_stem().unwrap().to_str().unwrap();
    let output = Command::new("java")
        .arg(&program_name)
        //how
        .stdin(File::open(format!("{}.in", file_stem)).unwrap())
        .output()
        .expect("run");
    // panics should probably just be a println
    // how? separate var?
    if output.status.code().unwrap() < 0 {
        //how
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Something is wrong with your OS: error {}", output.status);
    } else if output.status.code().unwrap() > 0 {
        //how
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Error compiling/running: {}", output.status);
    } else {
        fs::write(format!("{}.res", file_stem), &output.stdout)
            .expect("Unable to write output file");
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

}

#[async_std::main]
async fn main() -> Result<(), io::Error>{
    // read args provided to command from CLI
    let args: Vec<String> = env::args().collect();
    let test_dir: &Path;
    let program_name: PathBuf;
    let mut children = vec![];

    // obtain test_dir and program_name depending on the args provided
    // args.len() returns number of args including the executable name stored in &args[0]
    // move this match to a separate function?
    match args.len() {
        1 => {
            //assume everything is happening in current dir
            test_dir = Path::new(".");
            program_name = find_main_java(test_dir)?;
        }
        2 => {
            // java program is provided, tests are in current dir
            if args[1].ends_with(".java") {
                test_dir = Path::new(".");
                program_name = PathBuf::from(&args[1]);
            } else {
                // test dir is provided, java program is in current dir
                test_dir = Path::new(&args[1]);
                program_name = find_main_java(Path::new("."))?;
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
            panic!("too many arguments")
        }
    }

    let folder = fs::read_dir(test_dir).expect("error").into_iter();
    //contains all files, not just tests, but will be filtered later in for loop
    for file in folder {

        let file = match file {
            Ok(file) => file.path(),
            Err(error) => return Err(error),
        };

        match file.extension(){
            Some(extension) => {if extension == "in" {
                children.push(async_std::task::spawn({
                    let program_name = program_name.clone();
                    let file = file.clone();
                    //TODO which asyncs are necessary here?
                    async move {
                        let _ = thread(program_name, &file).await;
                    }}));
            }},
            None => {},
        }
    }
    for child in children {
        let _ = child.await;
    }
    return Ok(());
}
//for child in children {
//    let _ = child.await;
//}
