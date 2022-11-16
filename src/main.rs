use async_std;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{self, Error, ErrorKind, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn find_main_java(path: &PathBuf) -> Result<PathBuf, io::Error> {
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
        .arg(
            &program_name
                .file_name()
                .unwrap_or(OsStr::new(&program_name)),
        )
        .stdin(File::open(file)?)
        .output()?;
    let output_status = output.status.code().unwrap_or(-1);
    if output_status < 0 {
        io::stderr().write_all(&output.stderr)?;
        return Err(Error::new(
            ErrorKind::Other,
            format!("Error {} from java - program could not run", output_status),
        ));
    } else if output_status > 0 {
        io::stderr().write_all(&output.stderr)?;
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "Error {} from java - error compiling probably",
                output_status
            ),
        ));
    } else {
        fs::write(file.with_extension("res"), &output.stdout)?;

        let output_diff = Command::new("diff")
            .arg("--context")
            .arg(file.with_extension("res"))
            .arg(file.with_extension("out"))
            .output()?
            .stdout;
        if output_diff.is_empty() {
            //            println!("Thread {} fine", t_idx);
            return Ok(String::from(format!("\u{2705} Test {}", t_idx)));
        } else {
            fs::write(
                file.with_extension("diff"),
                String::from_utf8(output_diff)
                    .unwrap_or(String::from("Output and result files differ")),
            )?;
            //           println!("Thread {} \u{274C}", t_idx);
            return Ok(String::from(format!("\u{274C} Test {}", t_idx)));
        }
    }
}

async fn test_class(
    program_dir: &PathBuf,
    test_dir: &PathBuf,
) -> Result<Vec<String>, std::io::Error> {
    //compile all programs
    //test dir elements can be run with java without prior compilation, since they are only run once
    let program_name = PathBuf::from(".");
    let mut t_idx = 0;
    let mut children = vec![];
    let folder = fs::read_dir(test_dir)?.into_iter();
    for file in folder {
        let file = file?.path();
        if file.extension().unwrap_or(OsStr::new("")) == "in" {
            children.push(async_std::task::spawn({
                let program_name = program_name.clone().with_extension("");
                let file = file.clone();
                async move {
                    let a = thread(&t_idx, program_name, &file);
                    return a;
                }
            }));
            t_idx += 1;
        }
    }
    let mut outs = vec![];
    for child in children {
        let out = child.await.unwrap_or(String::from("Something went wrong"));
        outs.push(out);
    }
    Ok(outs)
}

async fn test_io(
    program_name: &PathBuf,
    test_dir: &PathBuf,
) -> Result<Vec<String>, std::io::Error> {
    let mut t_idx = 0;
    let mut children = vec![];
    let folder = fs::read_dir(test_dir)?.into_iter();
    for file in folder {
        let file = file?.path();
        if file.extension().unwrap_or(OsStr::new("")) == "in" {
            children.push(async_std::task::spawn({
                let program_name = program_name.clone().with_extension("");
                let file = file.clone();
                async move {
                    let a = thread(&t_idx, program_name, &file);
                    return a;
                }
            }));
            t_idx += 1;
        }
    }
    let mut outs = vec![];
    for child in children {
        let out = child.await.unwrap_or(String::from("Something went wrong"));
        outs.push(out);
    }
    return Ok(outs);
}

#[async_std::main]
async fn main() -> Result<(), io::Error> {
    // read args provided to command from CLI
    let args: Vec<String> = env::args().collect();
    let test_dir: PathBuf;
    let program_name: PathBuf;
    let mut children = vec![];

    // obtain test_dir and program_name depending on the args provided
    // args.len() returns number of args including the executable name stored in &args[0]
    // move this match to a separate function?
    match args.len() {
        1 => {
            //assume everything is happening in current dir
            test_dir = PathBuf::from(".");
            program_name = find_main_java(&test_dir)?;
        }
        //This is a special case, because it is a simple use-case and we might have multiple .java files
        2 => {
            // java program is provided, tests are in current dir
            if args[1].ends_with(".java") {
                test_dir = PathBuf::from(".");
                program_name = PathBuf::from(&args[1]);
            } else {
                // test dir is provided, java program is in current dir
                test_dir = PathBuf::from(&args[1]);
                program_name = find_main_java(&PathBuf::from("."))?;
            }
        }
        3 => {
            // java program and test dir are provided, order does not matter
            // should program name be exected without ".java" filetype
            program_name = PathBuf::from(&args[1]);
            test_dir = PathBuf::from(&args[2]);
        }
        _ => {
            panic!("too many arguments")
        }
    }

    if program_name.is_dir() {
        let mut relative_path = PathBuf::new();
        relative_path.push(&test_dir);
        let mut absolute_path = std::env::current_dir()?;
        absolute_path.push(relative_path);
        children = test_class(&program_name, &test_dir).await.unwrap();
    } else {
        let _output = Command::new("javac")
            .arg(&program_name.with_extension("java"))
            .output()?;
        children = test_io(&program_name, &test_dir).await.unwrap();
    }

    for child in children {
        println!("{}", child);
    }

    Ok(())
}
