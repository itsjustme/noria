extern crate clap;
extern crate nom_sql;
extern crate regex;

#[macro_use]
extern crate slog;
extern crate distributary;

use std::path::{Path, PathBuf};

fn traverse(path: &Path) -> Vec<PathBuf> {
    use std::fs;

    let mut files = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            files.push(path.clone())
        }
    }
    files
}

fn process_file(fp: &Path) -> Vec<String> {
    use regex::Regex;
    use std::io::Read;
    use std::fs::File;

    let e_re = "\\s+ [0-9]+ .+\t(.*)+";
    let q_re = "\\s+ [0-9]+ Query\t(.*)+";
    let entry_regex = Regex::new(e_re).unwrap();
    let query_regex = Regex::new(q_re).unwrap();

    let mut f = File::open(fp).unwrap();
    let mut s = String::new();

    f.read_to_string(&mut s).unwrap();

    let mut queries = Vec::new();
    let mut start = false;
    let mut capturing = false;
    let mut buffer = String::new();
    for l in s.lines() {
        if l.contains("### CHAIR PAPER LIST") {
            start = true;
        }

        if !start {
            continue;
        }

        if query_regex.is_match(&l) {
            if !buffer.is_empty() {
                queries.push(buffer.clone());
                buffer.clear();
            }
            for cap in query_regex.captures_iter(&l) {
                let qstr = &cap[1];
                buffer.push_str(qstr);
                buffer.push_str(" ");
                capturing = true;
            }
        } else if entry_regex.is_match(&l) && capturing {
            if !buffer.is_empty() {
                queries.push(buffer.clone());
                buffer.clear();
            }
            capturing = false;
        } else if capturing {
            buffer.push_str(&l);
            buffer.push_str(" ");
        }
    }
    queries
}

fn main() {
    use clap::{Arg, App};

    let log = distributary::logger_pls();

    let matches = App::new("process_paper_queries")
        .version("0.1")
        .about("Process extracted HotCRP paper queries.")
        .arg(
            Arg::with_name("source")
                .index(1)
                .help(
                    "Location of the HotCRP paper queries (in MySQL query log form).",
                )
                .required(true),
        )
        .get_matches();

    let path = matches.value_of("source").unwrap();

    let files = traverse(Path::new(path));

    for fname in files {
        info!(log, "Processing {:?}", fname);
        let queries = process_file(fname.as_path());

        for q in queries {
            match nom_sql::parse_query(&q) {
                Ok(_) => info!(log, "OK: {}", q),
                Err(e) => error!(log, "FAIL: {}: {:?}", q, e),
            }
        }
    }
}
