use std::{fs::File, io::{BufReader, BufRead, Read, Seek}};
use walkdir::{WalkDir, Error};
use clap::{Command, Arg, value_parser};
use regex::Regex;
use crossbeam_channel::unbounded;
use crossbeam_utils::thread::scope;

mod patterns;

struct SearchResult {
    path: String,
    lineno: usize,
    pattern_id: u32,
    text: String,

}

fn find_patterns(
    path: &String, p: &patterns::Pattern, lines: std::io::Lines<&mut BufReader<&mut File>>, results: &mut Vec<SearchResult>,
    max_text_len: usize
) {
    let re = Regex::new(p.pattern).unwrap();
    let pth = path.clone();
    for (lineidx, line) in lines.enumerate() {
        match line {
            Ok(l) => {
                if let Some(_) = re.captures(&l.to_lowercase()) {
                    let mut text = l.to_ascii_lowercase();
                    if text.len() > max_text_len {
                        text.truncate(max_text_len);
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

fn scan_file(path: &String, max_text_len: usize) -> Vec<SearchResult> {
    let file = File::open(path);
    let mut results = vec![];
    match file {
        Ok(mut f) => {
            for p in patterns::PYTHON_SUB_PATTERNS {
                let mut reader = BufReader::new(f.by_ref());
                find_patterns(path, p, reader.by_ref().lines(), &mut results, max_text_len);
                reader.seek(std::io::SeekFrom::Start(0)).unwrap();
            }
        }
        Err(err) => {
            println!("ERROR: {}", err)
        }
    }
    return results;
}

fn scan_files(target_dir: &String, worker_amount: u16, max_text_len: usize) {
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

        for _ in 0..worker_amount {
            let wsnd = wsnd.clone();
            scope.spawn(|_| {
                loop {
                    match frcv.recv() {
                        Ok(path) => {
                            for res in scan_file(&path, max_text_len) {
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
        .arg(Arg::new("target").long("target-dir").required(true))
        .arg(Arg::new("workers").long("workers-amount").help("amount of parallel workers")
            .value_parser(value_parser!(u16)).default_value("8"))
        .arg(Arg::new("max-len").long("maximum-text-length").help("maximum text length")
            .value_parser(value_parser!(usize)).default_value("100"))
        .get_matches();
    let worker_amount = *matches.get_one::<u16>("workers").unwrap();
    let max_text_len = *matches.get_one::<usize>("max-len").unwrap();

    let target_dir = matches.get_one::<String>("target").expect("required");
    scan_files(target_dir, worker_amount, max_text_len);
    return Ok(())
}
