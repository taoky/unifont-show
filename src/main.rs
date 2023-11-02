use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    process::exit,
};

use clap::Parser;

enum CharWidth {
    Width8,
    Width16,
}

struct Char {
    width: CharWidth,
    data: Vec<u16>,
}

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    inverted: bool,

    words: String,
}

fn main() {
    let args = Args::parse();

    let term_col = match termsize::get() {
        None => 80,
        Some(s) => s.cols,
    };
    if term_col < 16 {
        println!("terminal width must be at least 16");
        exit(1);
    }

    let hexfile = "./unifont_all-15.1.02.hex";
    let file = File::open(hexfile).unwrap();
    let mut mapping = HashMap::new();
    for line in BufReader::new(file).lines() {
        // split by :
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let (code, data) = line.split_once(':').unwrap();
        let code = u32::from_str_radix(code, 16).unwrap();
        let mut line_vec = vec![];
        let width = if data.len() == 64 {
            // push to line_vec every 4 chars
            for i in 0..16 {
                let byte = &data[i * 4..i * 4 + 4];
                let byte = u16::from_str_radix(byte, 16).unwrap();
                line_vec.push(byte);
            }
            CharWidth::Width16
        } else if data.len() == 32 {
            // push to line_vec every 2 chars
            for i in 0..16 {
                let byte = &data[i * 2..i * 2 + 2];
                let byte = u8::from_str_radix(byte, 16).unwrap();
                line_vec.push(byte as u16);
            }
            CharWidth::Width8
        } else {
            panic!("invalid data length: {}", data.len());
        };
        mapping.insert(
            code,
            Char {
                width,
                data: line_vec,
            },
        );
    }

    let mut start = 0;
    let words = args.words.chars().collect::<Vec<_>>();
    while start < words.len() {
        let mut end = start;
        let mut width = 0;
        while width < term_col {
            if end == words.len() {
                break;
            }
            let code = words[end] as u32;
            let char = mapping.get(&code).unwrap();
            let char_width = match char.width {
                CharWidth::Width8 => 8,
                CharWidth::Width16 => 16,
            };
            width += char_width;
            end += 1;
        }
        assert!(end <= words.len());
        for lno in 0..16 {
            for i in &words[start..end] {
                let code = *i as u32;
                let char = mapping.get(&code).unwrap();
                let line_vec = &char.data;
                let width = match char.width {
                    CharWidth::Width8 => 8,
                    CharWidth::Width16 => 16,
                };
                let line = line_vec[lno];
                let line = if width == 8 {
                    format!("{:08b}", line)
                } else {
                    format!("{:016b}", line)
                };
                if args.inverted {
                    print!("{}", line.replace("0", "█").replace("1", " "));
                } else {
                    print!("{}", line.replace("0", " ").replace("1", "█"));
                }
            }
            println!();
        }
        start = end;
    }
}
