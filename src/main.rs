use std::env;
use std::ffi::OsStr;
use async_std;
use std::fs;
use std::fs::File;
use std::io::{self, Write, Error, ErrorKind};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn find_main_java(path: &Path) -> Result<PathBuf, io::Error> {
    let paths = fs::read_dir(path)?.into_iter();
    for p in paths {
        let file = p?.path();
        if file.extension().unwrap_or(OsStr::new("")) == "java" {
            return Ok(file);
        }
    }
    return Err(Error::new(ErrorKind::Other, "Main java file not found."));
}

async fn thread(thread_number: u8, program_name: PathBuf, file: &Path) -> Result<String, io::Error> {
    let output = Command::new("java")
        .arg(&program_name)
        .stdin(File::open(file.with_extension("in"))?)
        .output()?;
    let output_status = output.status.code().unwrap_or(-1);
    if output_status < 0 {
        io::stderr().write_all(&output.stderr)?;
        return Err(Error::new(ErrorKind::Other, format!("Error {} from java - program could not run", output_status)));
    } else if output_status > 0 {
        io::stderr().write_all(&output.stderr)?;
        return Err(Error::new(ErrorKind::Other, format!("Error {} from java - error compiling probably", output_status)));
    } else {
        fs::write(file.with_extension("res"), &output.stdout)?;

        let out_file = file.with_extension("out");
        let res_file = file.with_extension("res");
        let output_diff = Command::new("diff")
            .arg("--context")
            .arg(&out_file)
            .arg(&res_file)
            .output()
            ?.stdout;
        if output_diff.is_empty() {
            println!("Thread {} fine", thread_number);
            return Ok(String::from(format!("{} done, fine",thread_number)));
        } else {
            println!("Thread {} NOT fine:\n {}", thread_number, 
                String::from_utf8(output_diff)
                .unwrap_or(String::from("Output and result files differ")));
                return Ok(String::from(format!("{} done, NOT fine", thread_number)));
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

    //contains all files, not just tests, but will be filtered later in for loop
    let folder = fs::read_dir(test_dir)?.into_iter();

    let mut thread_number = 1;

    for file in folder {
        let file = file?.path();
        if file.extension().unwrap_or(OsStr::new("")) == "in" {
            children.push(async_std::task::spawn({
                let program_name = program_name.clone();
                let file = file.clone();
                let thread_number = thread_number.clone();
                //TODO which asyncs are necessary here?
                async move {
                    let a = thread(thread_number, program_name, &file).await;
                    return a;
                }}));
            thread_number += 1;
        }
    }

    let mut idx = 0;
    let mut children_outputs = vec![];

    for child in children {
        children_outputs.push(child.await.unwrap_or(format!("{} likely did not finish", idx)));
        idx += 1;
    }

    for child in children_outputs{
        println!("Test {}", child);}
    Ok(())
}
