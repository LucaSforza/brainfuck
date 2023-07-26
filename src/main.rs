use std::{
    fs::File,
    io::{Bytes, Read},
    path::Path,
};

use clap::Parser;

const DEBUG: bool = false;

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

#[derive(Debug, Clone)]
enum Token {
    Output,
    Input,
    Move(isize),
    IncValue(isize),
    Loop(Vec<Token>),
}

fn compile(mut bytes: Bytes<File>, level: usize) -> (Vec<Token>, Bytes<File>) {
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        if let Some(byte) = bytes.next() {
            let char = byte.unwrap() as char;
            match char {
                '>' => tokens.push(Token::Move(1)),
                '<' => tokens.push(Token::Move(-1)),
                '+' => tokens.push(Token::IncValue(1)),
                '-' => tokens.push(Token::IncValue(-1)),
                '.' => tokens.push(Token::Output),
                ',' => tokens.push(Token::Input),
                '[' => {
                    let (inner_tokens, next_bytes) = compile(bytes, level + 1);
                    tokens.push(Token::Loop(inner_tokens));
                    bytes = next_bytes
                }
                ']' => {
                    if level == 0 {
                        eprintln!("closed an nonexistent loop");
                        std::process::exit(1);
                    } else {
                        return (tokens, bytes);
                    }
                }
                _ => (),
            }
        } else {
            if level > 0 {
                eprintln!("opened an nonexistent loop");
                std::process::exit(1);
            }
            break;
        }
    }

    (tokens, bytes)
}

fn optimize(tokens: Vec<Token>) -> Vec<Token> {
    let mut optimized: Vec<Token> = Vec::new();

    for token in tokens {
        match (optimized.last(), token) {
            (_, Token::Loop(sub_exp)) => {
                optimized.push(Token::Loop(optimize(sub_exp)));
            }

            (Some(&Token::IncValue(n)), Token::IncValue(v)) => {
                optimized.pop();
                optimized.push(Token::IncValue(n + v));
            }

            (Some(&Token::Move(n)), Token::Move(v)) => {
                optimized.pop();
                optimized.push(Token::Move(n + v))
            }

            (_, e) => optimized.push(e.clone()),
        }
    }
    optimized
}

fn run(memory: &mut Vec<i8>, dp: &mut isize, ip: &mut isize, tokens: &Vec<Token>) {
    for token in tokens {
        if DEBUG {
            println!("{:?}", *token)
        }
        match token {
            Token::Output => print!("{}", memory[*dp as usize] as u8 as char),
            Token::Input => {
                memory[*dp as usize] = match read_input() {
                    Ok(data) => data as i8,
                    Err(e) => {
                        eprintln!("error while reading data: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Token::Move(v) => *dp += v,
            Token::IncValue(v) => memory[*dp as usize] += *v as i8,
            Token::Loop(istr) => {
                while memory[*dp as usize] != 0 {
                    run(memory, dp, ip, istr)
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let path = Path::new(args.path.as_str());
    if !path.exists() {
        eprintln!("the path does not exists");
        std::process::exit(1);
    }

    let file = File::open(path).expect("the File can't be open or it is not a file");

    let mut memory: Vec<i8> = vec![0; 30_000];
    let mut dp: isize = 0;
    let mut ip: isize = 0;

    let tokens = optimize(compile(file.bytes(), 0).0);

    run(&mut memory, &mut dp, &mut ip, &tokens)
}
