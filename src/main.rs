use std::{fs::File, io::{BufReader, BufRead, Read, Seek}};
use walkdir::{WalkDir, Error};
use clap::{Command, Arg};
use regex::{Regex};
use crossbeam_channel::unbounded;
use crossbeam_utils::thread::scope;

mod patterns;

const WORKERS_AMOUNT: usize = 8;
const MAXIMUM_TEXT_LENGTH: usize = 100;

struct SearchResult {
    path: String,
    lineno: usize,
    pattern_id: u32,
    text: String,

}

fn find_patterns(
    path: &String, p: &patterns::Pattern, lines: std::io::Lines<&mut BufReader<&mut File>>, results: &mut Vec<SearchResult>
) {
    let re = Regex::new(p.pattern).unwrap();
    let pth = path.clone();
    for (lineidx, line) in lines.enumerate() {
        match line {
            Ok(l) => {
                if let Some(_) = re.captures(&l.to_lowercase()) {
                    let mut text = l.to_ascii_lowercase();
                    if text.len() > MAXIMUM_TEXT_LENGTH {
                        text.truncate(MAXIMUM_TEXT_LENGTH);
                    }
                    let result = SearchResult{
                        path: (*pth).to_string(), pattern_id: p.id, lineno: lineidx + 1, text: text
                    };
                    results.push(result);
                }
            },
            _ => {}
        }
    }
}

fn scan_file(path: &String) -> Vec<SearchResult> {
    let file = File::open(path);
    let mut results = vec![];
    match file {
        Ok(mut f) => {
            for p in patterns::PYTHON_SUB_PATTERNS {
                let mut reader = BufReader::new(f.by_ref());
                find_patterns(path, p, reader.by_ref().lines(), &mut results);
                reader.seek(std::io::SeekFrom::Start(0)).unwrap();
            }
        }
        Err(err) => {
            println!("ERROR: {}", err)
        }
    }
    return results;
}

fn scan_files(target_dir: &String) {
    let (fsnd, frcv) = unbounded();
    let (wsnd, wrcv) = unbounded::<SearchResult>();

    scope(|scope| {
        scope.spawn(|_| {
            loop {
                match wrcv.recv() {
                    Ok(result) => {
                        println!("{} found. File: {}, line: {}, text: {}",
                        result.pattern_id, result.path, result.lineno, result.text)
                    },
                    Err(_) => break,
                }
            }
        });

        for _ in 0..WORKERS_AMOUNT {
            let wsnd = wsnd.clone();
            scope.spawn(|_| {
                loop {
                    match frcv.recv() {
                        Ok(path) => {
                            for res in scan_file(&path) {
                                wsnd.send(res).unwrap();
                            }
                        },
                        Err(_) => break,
                    }
                }
                drop(wsnd);
            });
        }
        drop(wsnd);

        for entry in WalkDir::new(target_dir) {
            match entry {
                Ok(e) => {
                    let path = String::from(e.path().as_os_str().to_str().unwrap());
                    if path.ends_with(".py") {
                        fsnd.send(path).unwrap();
                    }
                },
                _ => (),
            }
        }
        drop(fsnd);
    }).unwrap();
    
}

fn main() -> Result<(), Error> {
    let matches = Command::new("xgrep")
        .arg(Arg::new("target-dir").long("target-dir").required(true)
        ).get_matches();

    let target_dir = matches.get_one::<String>("target-dir").expect("required");
    scan_files(target_dir);
    return Ok(())
}
