use core::panic;
use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

struct Args {
    /// --file: The input file to compress / decompress based on the --compress / --decompress flags
    file: Option<String>,

    /// -- compress: indicates that the data in should be compressed
    compress: bool,

    /// -- decompress: indicates that the data in should be decompressed
    decompress: bool,
}

struct GzipHeader {
    id1: u8,
    id2: u8,
    cm: u8,
    flg: u8,
    mtime: u32,
    xfl: u8,
    os: u8,
}

fn main() {
    let mut args = Args {
        file: None,
        compress: false,
        decompress: false,
    };

    let to_parse: Vec<String> = env::args().collect();

    to_parse.iter().enumerate().for_each(|(i, f)| {
        if f == "--compress" {
            args.compress = true;
        }

        if f == "--decompress" {
            args.decompress = true;
        }

        if f == "--file" {
            if to_parse.len() - 1 < i + 1 {
                eprintln!("Please specify a file path!");
                process::exit(1);
            }

            let path = to_parse[i + 1].clone();
            args.file = Some(path);
        }
    });

    if args.file.is_none() {
        eprintln!("A file must be specified");
        process::exit(1);
    }

    if args.compress {
        compress(args.file.unwrap());
    } else {
        decompress();
    }
}

fn compress(path: String) {
    let f = match File::open(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error reading file {}", e),
    };

    let out_path = format!("{}.gz", path);
    let out = match File::create_new(out_path) {
        Ok(f) => f,
        Err(e) => panic!("Error creating destination {}", e),
    };

    let header = GzipHeader {
        id1: 0x1F,
        id2: 0x8B,
        cm: 0x08,
        flg: 0x00,
        mtime: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32,
        xfl: 0x00,
        os: 0x07,
    };

    let mut writer = BufWriter::new(out);

    writer
        .write_all(&[header.id1, header.id2, header.flg])
        .expect("Something went wrong while compressing");
    writer
        .write_all(&header.mtime.to_le_bytes())
        .expect("Something went wrong while compressing");
    writer
        .write_all(&[header.xfl, header.os])
        .expect("Something went wrong while compressing");

    deflate(&f, &mut writer)
}

fn deflate(file: &File, writer: &mut BufWriter<File>) {
    lz77_encoding(file);
}

fn lz77_encoding(file: &File) {
    let test = "AABCBBABC";

    let search_buff: &[u8];
    let lookahead_buff: &[u8];

    for c in test.chars() {
        let b = c.to_string().as_bytes();

        for (i, in_buf) in search_buff.iter().enumerate() {
            if b == in_buf {
            } else {
                println!("({}, {}, {})",)
            }
        }
    }
}

fn decompress() {}
