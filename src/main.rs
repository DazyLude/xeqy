use dialoguer::Input;
use rand::{thread_rng, Rng};
use xeqy::ConfigData;

const HELP_MESSAGE: &str = r#"
This game is called x=y. It's goal is to solve a randomly generated puzzle.
You need to rearrange an x number of digits, using basic arithmetic operations and parentheses, to make an expression that equals to y.
For example, for a given prompt "1 2 3 4 = 10", all of "1+2+3+4", "3*4-1*2" and "(2*3+4)^1" are correct answers.

"#;

fn main() {
    // loading config
    let mut config = ConfigData::load_config();
    let mut puzz = Puzzle {
        answer: 0,
        prompt: "".to_string(),
    };
    println!("This is Xeqy!\nType help to display help message, or type start to begin the game!");

    let mut run = true;
    let mut waiting_for_an_answer = false;
    while run {
        let input = Input::<String>::new().interact_text().unwrap();
        match input.as_str() {
            "exit" => run = false,
            "help" => println!("{}", HELP_MESSAGE),
            "start" => {
                println!(
                    "Your current score is: {}, amount of digits is {}",
                    config.score, config.x
                );
                puzz = generate_puzzle(config.x);
                println!("{}", puzz.prompt);
                waiting_for_an_answer = true;
            }
            _ => {
                if waiting_for_an_answer {
                    match deparenthesizer(&input.to_string()) {
                        Ok(value) => {
                            if value == puzz.answer {
                                println!("victory!");
                                config.score += config.x as i64;
                                waiting_for_an_answer = false;
                            } else {
                                println!("{value}")
                            }
                        }
                        Err(msg) => println!(
                            "I can't understand your input, there is something wrong:\n{msg}"
                        ),
                    }
                } else {
                    println!("Unknown command. Try again!")
                }
            }
        }
    }

    // generating a config file with a new score
    config.save_config().unwrap();
    println!("Bye!");
}

struct Puzzle {
    answer: i64,
    prompt: String,
}

fn generate_puzzle(x: u32) -> Puzzle {
    let mut rng = thread_rng();
    let mut max_size: u32;
    let mut size: u32 = 0;
    let mut values: Vec<i64> = Vec::new();
    // generating values to do math magic with
    while size < x {
        max_size = if (x / 2) <= (x - size) {
            x / 2
        } else {
            x - size
        };
        let add_size = rng.gen_range(0..max_size) + 1;
        if add_size == 1 {
            values.push(rng.gen_range(0..10));
        } else {
            values.push(
                rng.gen_range(10_u32.pow(add_size - 1)..10_u32.pow(add_size))
                    .into(),
            );
        }
        size += add_size;
    }

    let mut prompt: String = "".to_string();
    for value in &values {
        for digit in value.to_string().chars() {
            prompt.push(digit);
            prompt.push(' ');
        }
    }

    let mut expression = values.pop().unwrap().to_string();
    for value in values {
        expression += match rng.gen_range(0..10) {
            x if x < 3 => "+",
            x if x < 6 => "-",
            x if x < 8 => "*",
            x if x < 10 => "/",
            _ => "^", // :^)
        };
        expression += &value.to_string();
    }

    // generating an expression
    let answer = simexer(&expression).unwrap();
    prompt += "= ";
    prompt += &answer.to_string();
    Puzzle { answer, prompt }
}

#[derive(PartialEq, Ord, PartialOrd, Eq)]
enum Operators {
    Pow,
    Mul,
    Div,
    Add,
    Sub,
}

// Parsers
// simple expression parser (simexer): value operator value operator value ...
// always starts with a digit, followed by 0 or more digits,
// followed by an operator, followed by 1 or more digits, and repeat until EOL
// whitespace characters are ignored
// intermediate result: vector of i64 and vector of operators in form of an enum
// values:      1st  2nd  3rd...
// operators:      1st  2nd  ...
// an expression can start with an unary - operator, then the first value is multilpied by -1
// these vectors are processed by the calculator
// return value: i64

fn simexer(input: &str) -> Result<i64, ()> {
    let mut vals: Vec<i64> = Vec::new();
    let mut ops: Vec<Operators> = Vec::new();
    let mut buf: String = String::new();

    // why not filter whitespaces from the very beginning
    let mut chars = input.chars().filter(|char| !char.is_whitespace());
    let radix = 10;

    let mut starts_with_unary_sub = false;
    let mut recording_value = true;

    // catches unary - operator
    match chars.next() {
        Some('-') => starts_with_unary_sub = true,
        Some(digit) => {
            if digit.is_digit(radix) {
                buf.push(digit)
            } else {
                return Err(());
            }
        }
        None => return Err(()),
    };

    while let Some(next) = chars.next() {
        if recording_value ^ next.is_digit(radix) {
            if recording_value {
                vals.push(match buf.parse::<i64>() {
                    Ok(val) => val,
                    Err(_) => return Err(()),
                });
            } else {
                ops.push(match buf.as_str() {
                    "^" | "pow" => Operators::Pow,
                    "*" => Operators::Mul,
                    "/" => Operators::Div,
                    "+" => Operators::Add,
                    "-" => Operators::Sub,
                    _ => return Err(()),
                });
            }
            recording_value = !recording_value;
            buf.clear();
        }
        buf.push(next);
    }
    // when iterator ends, it doesn't deposit last value
    vals.push(match buf.parse::<i64>() {
        Ok(val) => val,
        Err(_) => return Err(()),
    });

    if vals.len() != ops.len() + 1 {
        return Err(());
    }
    if starts_with_unary_sub {
        vals[0] = vals[0] * -1;
    }

    let mut parse_operator =
        |opr: Operators, opn: fn(i64, i64) -> i64, cond: fn(i64, i64) -> bool| -> Result<(), ()> {
            while ops.contains(&opr) {
                let index: usize = ops.binary_search(&opr).unwrap();
                if !cond(vals[index], vals[index + 1]) {
                    return Err(());
                }
                vals[index] = opn(vals[index], vals.remove(index + 1_usize));
                ops.remove(index);
            }
            Ok(())
        };

    parse_operator(
        Operators::Pow,
        |x, y| x.pow(y.try_into().unwrap()),
        |_x, _y| true,
    )?;
    parse_operator(Operators::Mul, |x, y| x * y, |_x, _y| true)?;
    parse_operator(Operators::Div, |x, y| x / y, |x, y| x % y == 0)?;
    parse_operator(Operators::Add, |x, y| x + y, |_x, _y| true)?;
    parse_operator(Operators::Sub, |x, y| x - y, |_x, _y| true)?;
    let output = vals[0];
    return Ok(output);
}

// deparenthesizer: processes expressions with parentheses, opening them.
// first it measures "depth" of an expression
// expressions with the highest priority are parsed the first
// then the value is inserted into the expression
//
// returns a value of an expression or an error
// error is an expression that failed to parse by simexpr, or a message about wrong parentheses

fn deparenthesizer(input: &String) -> Result<i64, &str> {
    let mut temp: String = input.to_string().clone();
    while temp.contains('(') & temp.contains(')') {
        let mut chars = input.chars();
        let mut index = 0;
        let mut last_op = 0;
        let mut last_cl = 0;
        while let Some(next) = chars.next() {
            match next {
                '(' => last_op = index,
                ')' => {
                    last_cl = index;
                    break;
                }
                _ => {}
            }
            index += 1;
        }
        let mut expression = temp.clone();
        expression = expression.get(last_op + 1..last_cl).unwrap().to_string();
        let value = match simexer(&expression) {
            Ok(val) => val,
            Err(_) => return Err(input),
        };
        let cut = temp.get(..last_op).unwrap().to_owned();
        let end = temp.get(last_cl + 1..).unwrap();

        temp = cut + &value.to_string() + end;
    }
    if temp.contains('(') | temp.contains(')') {
        if temp.contains('(') {
            return Err("Unmatched parentheses! More ( than )");
        } else {
            return Err("Unmatched parentheses! More ) than (");
        }
    }
    match simexer(&temp) {
        Ok(val) => return Ok(val),
        Err(_) => return Err(input),
    }
}
