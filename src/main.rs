use std::{fs::File, io::Read, path::Path};

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    /// the path to the brainfuck file
    path: String,
}

fn read_input() -> Result<u8, std::io::Error> {
    let mut input = [0];
    std::io::stdin().read(&mut input)?;
    Ok(input[0])
}

fn main() {
    let args = Args::parse();
    let path = Path::new(args.path.as_str());
    if !path.exists() {
        eprintln!("the path does not exists");
        std::process::exit(1);
    }

    let debug = false;

    let file = File::open(path).expect("the File can't be open or it is not a file");

    let mut memory: Vec<i8> = vec![0; 30_000];
    let mut dp: usize = 0;
    let mut ip: usize = 0;

    let bytes: Vec<u8> = file.bytes().map(|b| b.unwrap()).collect();

    loop {
        if ip >= bytes.len() {
            break;
        }
        let byte = bytes[ip] as char;

        if debug {
            println!("istruction: {}", byte);
            println!("data ptr: {}", dp);
            memory.iter().enumerate().for_each(|(i, v)| {
                if *v != 0 {
                    println!("{}: {}", i, v)
                }
            })
        }

        match byte {
            '>' => dp += 1,
            '<' => dp -= 1,
            '+' => memory[dp] += 1,
            '-' => memory[dp] -= 1,
            '.' => print!("{}", memory[dp] as u8 as char),
            '[' => {
                if memory[dp] == 0 {
                    let mut skip: usize = 0;
                    loop {
                        ip += 1;
                        let r = bytes[ip] as char;
                        if r == ']' {
                            if skip == 0 {
                                break;
                            }
                            skip -= 1
                        } else if r == '[' {
                            skip += 1
                        }
                    }
                }
            }
            ']' => {
                if memory[dp] != 0 {
                    let mut skip: usize = 0;
                    loop {
                        ip -= 1;
                        let r = bytes[ip] as char;
                        if r == '[' {
                            if skip == 0 {
                                break;
                            }
                            skip -= 1
                        } else if r == ']' {
                            skip += 1
                        }
                    }
                }
            }
            ',' => {
                memory[dp] = match read_input() {
                    Ok(data) => data as i8,
                    Err(e) => {
                        eprintln!("error while reading data: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            _ => {}
        }
        ip += 1;
    }
}
