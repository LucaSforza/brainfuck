use std::{
    env::{self, Args},
    fs,
    io::{Read, Write},
    path::Path,
};

const MEMORY_SIZE: usize = 30_000;

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
                                "\n\n[RUNTIME ERROR] error while reading data\nMessage error: {}",
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
                Token::IncValue(v) => {
                    self.write_memory(self.read_memory().wrapping_add_signed(*v as i8))
                }
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
    std::io::stdout().flush()?;
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

fn search_extra_loop(stack: &mut Vec<(char, u32, u32)>) -> (u32, u32) {
    let mut level: u32 = 0;
    loop {
        if let Some(values) = stack.pop() {
            match values {
                ('[', line, position) => {
                    if level == 0 {
                        return (line, position);
                    }
                    level -= 1
                }
                (']', _, _) => level += 1,
                _ => panic!("token unespected in the stack: {:?}", values.0),
            }
        } else {
            panic!("The extra loop was not found")
        }
    }
}

fn compile<'a>(
    mut bytes: std::slice::Iter<'a, u8>,
    level: usize,
    stack: &mut Vec<(char, u32, u32)>,
    line: &mut u32,
    position: &mut u32,
) -> (Vec<Token>, std::slice::Iter<'a, u8>) {
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        if let Some(byte) = bytes.next() {
            let char = *byte as char;
            match char {
                '\n' => {
                    *line += 1;
                    *position = 0
                }
                '>' => tokens.push(Token::Move(1)),
                '<' => tokens.push(Token::Move(-1)),
                '+' => tokens.push(Token::IncValue(1)),
                '-' => tokens.push(Token::IncValue(-1)),
                '.' => tokens.push(Token::Output),
                ',' => tokens.push(Token::Input),
                '[' => {
                    stack.push(('[', *line, *position));
                    let (inner_tokens, next_bytes) =
                        compile(bytes, level + 1, stack, line, position);
                    tokens.push(Token::Loop(inner_tokens));
                    bytes = next_bytes
                }
                ']' => {
                    if level == 0 {
                        eprintln!(
                            "[COMPILE ERROR] closed an nonexistent loop on line: {}:{}",
                            line, position
                        );
                        std::process::exit(1);
                    } else {
                        stack.push((']', *line, *position));
                        return (tokens, bytes);
                    }
                }
                _ => (),
            }
        } else {
            if level > 0 {
                let (line, position) = search_extra_loop(stack);
                eprintln!(
                    "[COMPILE ERROR] opened an nonexistent loop on line: {}:{}",
                    line, position
                );
                std::process::exit(1);
            }
            break;
        }
        *position += 1
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

struct Config {
    file_path: String,
}

impl Config {
    fn parse_args(mut args: Args) -> Self {
        let program = args.next().unwrap();
        let file_path = args.next().unwrap_or_else(|| {
            eprintln!("[ERROR] no path to the brainfuck file provided");
            eprintln!("[INFO] Usage: {program} <brainfuck file path>");
            std::process::exit(1);
        });
        Self {
            file_path: file_path,
        }
    }
}

fn main() {
    let config = Config::parse_args(env::args());
    let path = Path::new(config.file_path.as_str());
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

    let tokens = optimize(compile(data.as_bytes().iter(), 0, &mut Vec::new(), &mut 1, &mut 1).0);

    interpreter.run(&tokens)
}
