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
        if file.extension().unwrap_or(OsStr::new(".")) == "java" {
            return Ok(file);
        }
    }
    return Err(Error::new(ErrorKind::Other, "Main java file not found."));
}

fn thread(t_idx: &u8, program_name: PathBuf, file: &Path) -> Result<String, io::Error> {
    //this should be simpler
    let output = Command::new("java")
        .arg("-cp")
        .arg(&program_name.parent().unwrap_or(Path::new(".")))
        //this probably needs a better solution than OsStr::new
        .arg(&program_name.file_name().unwrap_or(OsStr::new(&program_name)))
        .stdin(File::open(file)?)
        .output()?;
    let output_status = output.status.code().unwrap_or(-1);
    if output_status < 0 {
        io::stderr().write_all(&output.stderr)?;
        return Err(Error::new(ErrorKind::Other,
                format!("Error {} from java - program could not run", output_status)));
    } else if output_status > 0 {
        io::stderr().write_all(&output.stderr)?;
        return Err(Error::new(ErrorKind::Other,
                format!("Error {} from java - error compiling probably", output_status)));
    } else {
        fs::write(file.with_extension("res"), &output.stdout)?;

        let output_diff = Command::new("diff")
            .arg("--context")
            .arg(file.with_extension("out"))
            .arg(file.with_extension("res"))
            .output()
            ?.stdout;
        if output_diff.is_empty() {
//            println!("Thread {} fine", t_idx);
            return Ok(String::from(format!("\u{2705} Test {}",t_idx)));
        } else {
            fs::write(file.with_extension("diff"), String::from_utf8(output_diff)
                .unwrap_or(String::from("Output and result files differ")))?;
 //           println!("Thread {} \u{274C}", t_idx);
            return Ok(String::from(format!("\u{274C} Test {}", t_idx)));
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
                program_name = PathBuf::from(&args[1]);
                test_dir = Path::new(&args[2]);
        }
        _ => {
            panic!("too many arguments")
        }
    }

    //contains all files, not just tests, but will be filtered later in for loop
    let folder = fs::read_dir(test_dir)?.into_iter();
    let _output = Command::new("javac")
        .arg(&program_name.with_extension("java"))
        .output()?;
    

    let mut t_idx = 1;

    for file in folder {
        let file = file?.path();
        if file.extension().unwrap_or(OsStr::new("")) == "in" {
            children.push(async_std::task::spawn({
                let program_name = program_name.clone().with_extension("");
                let file = file.clone();
                //TODO which asyncs are necessary here?
                async move {
                    let a = thread(&t_idx, program_name, &file);
                    return a;
                }}));
            t_idx += 1;
        }
    }

    let mut idx = 0;
    let mut children_outputs = vec![];

    for child in children {
        children_outputs.push(child.await.unwrap_or(format!("{} \u{2753} - likely did not finish", idx)));
        idx += 1;
    }

    for child in children_outputs{
        println!("{}", child);}
    Ok(())
}

