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

fn io_thread(t_idx: &u8, program_name: &PathBuf, file: &Path) -> Result<String, io::Error> {
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
    return handle_diffs(file, &output, t_idx);
}

fn class_thread(
    t_idx: &u8,
    program_name: &PathBuf,
    file: &Path,
    program_dir: &PathBuf,
) -> Result<String, io::Error> {
    let output = Command::new("java")
        .current_dir(program_dir)
        .arg(&program_name)
        .output()?;
    return handle_diffs(file, &output, t_idx);
}

fn handle_diffs(
    file: &Path,
    output: &std::process::Output,
    t_idx: &u8,
) -> Result<String, io::Error> {
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
            return Ok(String::from(format!("\u{2705} Test {}", t_idx)));
        } else {
            fs::write(
                file.with_extension("diff"),
                String::from_utf8(output_diff)
                    .unwrap_or(String::from("Output and result files differ")),
            )?;
            return Ok(String::from(format!("\u{274C} Test {}", t_idx)));
        }
    }
}

async fn test_class(
    program_dir: &PathBuf,
    test_dir: &PathBuf,
) -> Result<Vec<String>, std::io::Error> {
    let mut t_idx = 0;
    let mut children = vec![];
    let folder = fs::read_dir(test_dir)?.into_iter();
    let programs = fs::read_dir(program_dir)?.into_iter();

    //compile all programs
    for program in programs {
        let p = program?.path();
        if p.extension().unwrap_or(OsStr::new("")) == "java" {
            let _output = Command::new("javac").arg(p).output()?;
        }
    }

    //test dir elements can be run with java without prior compilation, since they are only run once
    for file in folder {
        let file = &file?.path();
        if file.extension().unwrap_or(OsStr::new("")) == "java" {
            children.push(async_std::task::spawn({
                let program_dir = program_dir.clone();
                let mut program_name = std::env::current_dir()?;
                program_name.push(file);
                let mut file_name = std::env::current_dir()?;
                file_name.push(
                    &file
                        .file_name()
                        .expect("No test file")
                        .to_str()
                        .expect("Test program name broken/too short")
                        .to_lowercase(),
                );

                async move {
                    let a = class_thread(&t_idx, &program_name, &file_name, &program_dir);
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
                    let a = io_thread(&t_idx, &program_name, &file);
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
    let children: Vec<String>;

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
            // should program name be exected without ".java" filetype
            program_name = PathBuf::from(&args[1]);
            test_dir = PathBuf::from(&args[2]);
        }
        _ => {
            panic!("too many arguments")
        }
    }

    if program_name.is_dir() {
        children = test_class(&program_name, &test_dir)
            .await
            .expect("tests failed");
    } else {
        let _output = Command::new("javac")
            .arg(&program_name.with_extension("java"))
            .output()?;
        children = test_io(&program_name, &test_dir)
            .await
            .expect("tests failed");
    }

    for child in children {
        println!("{}", child);
    }

    Ok(())
}
