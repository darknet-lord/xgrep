use std::{fs::File, io::{BufReader, BufRead, Read, Seek}};
use walkdir::{WalkDir, Error};
use clap::{Command, Arg};
use regex::{Regex};
mod patterns;


fn find_patterns(p: &patterns::Pattern, lines: std::io::Lines<&mut BufReader<&mut File>>) {
    let re = Regex::new(p.pattern).unwrap();
    for (lineno, line) in lines.enumerate() {
        match line {
            Ok(l) => {
                if let Some(_) = re.captures(&l.to_lowercase()) {
                    println!("FOUND {}: line={}, description={}, text={}", p.name, lineno, p.description, l)
                }
            },
            _ => {}
        }
    }
}

fn scan_file(path: &str) {
    let file = File::open(path);
    match file {
        Ok(mut f) => {
            for p in patterns::PYTHON_SUB_PATTERNS {
                let mut reader = BufReader::new(f.by_ref());
                find_patterns(p, reader.by_ref().lines());
                reader.seek(std::io::SeekFrom::Start(0)).unwrap();
            }
        }
        Err(err) => {
            println!("ERROR: {}", err)
        }
    }
}

fn scan_files(target_dir: &String) {
    for entry in WalkDir::new(target_dir) {
        match entry {
            Ok(e) => {
                let path = e.path().as_os_str().to_str().unwrap();
                if path.ends_with(".py") {
                    scan_file(path);
                }
            },
            _ => {}
        }
        
    }
}

fn main() -> Result<(), Error> {
    let matches = Command::new("xgrep")
        .arg(Arg::new("target-dir").long("target-dir").required(true)
        ).get_matches();

    let target_dir = matches.get_one::<String>("target-dir").expect("required");
    scan_files(target_dir);
    return Ok(())
}
