use std::{fs, io::Read, path::Path};

use clap::Parser;

const MEMORY_SIZE: usize = 30_000;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    /// the path to the brainfuck file
    path: String,
}

struct Interpreter {
    memory: Vec<u8>,
    dp: usize, // Data Pointer
}
impl Interpreter {
    fn write_memory(&mut self, byte: u8) {
        self.memory[self.dp] = byte
    }
    fn read_memory(&self) -> u8 {
        self.memory[self.dp]
    }
    fn run(&mut self, istructions: &Vec<Token>) {
        for token in istructions {
            match token {
                Token::Output => print!("{}", self.read_memory() as char),
                Token::Input => {
                    self.memory[self.dp] = match read_input() {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!(
                                "[RUNTIME ERROR] error while reading data\nMessage error: {}",
                                e
                            );
                            std::process::exit(1);
                        }
                    }
                }
                Token::Move(v) => match self.dp.checked_add_signed(*v) {
                    Some(mut new_dp) => {
                        if new_dp >= MEMORY_SIZE {
                            // Overflow is not possible
                            // The only way an overflow is possible is if there are enough '+'s in a row in the brainfuck file
                            // (so it doesn't happen using loops) to cause an overflow with the usize type, but this is highly unlikely
                            new_dp -= MEMORY_SIZE
                        }
                        self.dp = new_dp
                    }
                    None => {
                        // Underflow
                        let v = -v as usize;
                        let new_dp = v - self.dp;
                        self.dp = MEMORY_SIZE - new_dp;
                    }
                },
                Token::IncValue(v) => match self.read_memory().checked_add_signed(*v as i8) {
                    Some(data) => self.write_memory(data),
                    None => {
                        let v = *v as usize;
                        let result = self.read_memory() as usize;
                        self.write_memory((v + result) as u8)
                    }
                },
                Token::Loop(istr) => {
                    while self.read_memory() != 0 {
                        self.run(istr)
                    }
                }
            }
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            memory: vec![0; MEMORY_SIZE],
            dp: 0,
        }
    }
}

fn read_input() -> Result<u8, std::io::Error> {
    let mut input = [0];
    std::io::stdin().read(&mut input)?;
    Ok(input[0])
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Output,
    Input,
    Move(isize),
    IncValue(isize),
    Loop(Vec<Token>),
}

fn compile(
    mut bytes: std::slice::Iter<'_, u8>,
    level: usize,
) -> (Vec<Token>, std::slice::Iter<'_, u8>) {
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        if let Some(byte) = bytes.next() {
            let char = *byte as char;
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
                        eprintln!("[COMPILE ERROR] closed an nonexistent loop");
                        std::process::exit(1);
                    } else {
                        return (tokens, bytes);
                    }
                }
                _ => (),
            }
        } else {
            if level > 0 {
                eprintln!("[COMPILE ERROR] opened an nonexistent loop");
                std::process::exit(1);
            }
            break;
        }
    }

    (tokens, bytes)
}

fn optimize(tokens: Vec<Token>) -> Vec<Token> {
    let mut optimized: Vec<Token> = Vec::with_capacity(tokens.len());

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

fn main() {
    let args = Args::parse();
    let path = Path::new(args.path.as_str());
    if !path.exists() {
        eprintln!("[ERROR] the path does not exists");
        std::process::exit(1);
    }

    let data = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!(
            "[ERROR]the File can't be open or it is not a file\nError message: {}",
            e,
        );
        std::process::exit(1)
    });

    let mut interpreter = Interpreter::default();

    let tokens = optimize(compile(data.as_bytes().iter(), 0).0);

    interpreter.run(&tokens)
}
