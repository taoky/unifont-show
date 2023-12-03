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
    /// Show black background instead of white
    #[clap(short, long)]
    inverted: bool,

    /// Stream mode. Takes input from stdin and prints it as it comes.
    #[clap(short, long)]
    stream: bool,

    /// Words to print. In stream mode, this is ignored.
    #[clap(default_value = "")]
    words: String,
}

macro_rules! get_term_col {
    () => {
        match termsize::get() {
            None => 80,
            Some(s) => {
                if s.cols < 16 {
                    println!("terminal width must be at least 16");
                    exit(1);
                }
                s.cols
            }
        }
    };
}

fn push_line(words: &[char], mapping: &HashMap<u32, Char>, args: &Args, initial_term_col: u16) {
    let term_col = if args.stream {
        get_term_col!()
    } else {
        initial_term_col
    };
    let mut start = 0;
    while start < words.len() {
        let mut end = start;
        let mut width = 0;
        while width < term_col {
            // check while it still contains some space
            if end == words.len() {
                // reached end of string
                break;
            }
            let code = words[end] as u32;
            let char = mapping.get(&code).unwrap();
            let char_width = match char.width {
                CharWidth::Width8 => 8,
                CharWidth::Width16 => 16,
            };
            // don't push if it exceeds terminal width
            if width + char_width > term_col {
                break;
            }
            width += char_width;
            // here it's safe to push -- we make sure that
            // [start, end) does not exceed terminal width
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
                    print!("{}", line.replace('0', "█").replace('1', " "));
                } else {
                    print!("{}", line.replace('0', " ").replace('1', "█"));
                }
            }
            println!();
        }
        start = end;
    }
}

fn main() {
    let args = Args::parse();

    let initial_term_col = get_term_col!();

    let hexfile = "./unifont_all.hex";
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

    if !args.stream {
        let words = args.words.chars().collect::<Vec<_>>();
        push_line(&words, &mapping, &args, initial_term_col);
    } else {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let words = line.chars().collect::<Vec<_>>();
            push_line(&words, &mapping, &args, initial_term_col);
        }
    }
}
