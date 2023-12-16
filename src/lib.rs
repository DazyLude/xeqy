use std::path::Path;

const MAX_SIZE: u32 = 2;
const CONFIG_LENGTH: usize = 16;
const CONFIG_PATH: &str = ".config";

// Config stores score and settings
pub struct ConfigData {
    pub max_size: u32,
    pub x: u32,
    pub score: i64,
}

impl ConfigData {
    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        for byte in self.x.to_be_bytes() {
            data.push(byte)
        }
        for byte in self.max_size.to_be_bytes() {
            data.push(byte)
        }
        for byte in self.score.to_be_bytes() {
            data.push(byte)
        }
        debug_assert!(data.len() == CONFIG_LENGTH);
        data
    }
    pub fn save_config(&self) -> Result<(), std::io::Error> {
        std::fs::write(Path::new(CONFIG_PATH), self.serialize())?;
        Ok(())
    }

    pub fn deserialize(data: Vec<u8>) -> Self {
        if data.len() != CONFIG_LENGTH {
            println!("trying to deserialize byte sequence of length {}, length of {} was expected. Loading default config.",
                data.len(),
                CONFIG_LENGTH
            );
            return ConfigData {
                x: 4,
                max_size: 2,
                score: 0,
            };
        }
        ConfigData {
            x: u32::from_be_bytes(data[0..4].try_into().unwrap()),
            max_size: u32::from_be_bytes(data[4..8].try_into().unwrap()),
            score: i64::from_be_bytes(data[8..16].try_into().unwrap()),
        }
    }
    pub fn load_config() -> ConfigData {
        let config_path = Path::new(CONFIG_PATH);
        if config_path.exists() {
            // loads an existing config
            return ConfigData::deserialize(std::fs::read(config_path).unwrap());
        } else {
            // or loads default values
            return ConfigData {
                x: 4,
                max_size: 2,
                score: 0,
            };
        };
    }
}

// the game part

pub mod you_know_the_rules {
    use crate::ConfigData;

    use rand::thread_rng;
    use rand::Rng;

    pub struct Puzzle {
        pub answer: i64,
        pub prompt: String,
        pub expression: String,
    }

    pub fn generate_puzzle(config: &ConfigData) -> Puzzle {
        let mut rng = thread_rng();
        let mut max_size: u32;
        let mut size: u32 = 0;
        let mut values: Vec<i64> = Vec::new();
        // generating values to do math magic with
        while size < config.x {
            max_size = config.max_size.min(config.x - size);
            let add_size = rng.gen_range(1..max_size + 1);
            values.push(
                rng.gen_range(10_u32.pow(add_size - 1)..10_u32.pow(add_size))
                    .into(),
            );
            size += add_size;
        }

        let mut prompt: String = "".to_string();
        let mut sorted: Vec<char> = Vec::new();
        for value in &values {
            for digit in value.to_string().chars() {
                sorted.push(digit);
            }
        }
        sorted.sort();
        for digit in sorted.clone() {
            prompt.push(digit);
            prompt.push(' ');
        }

        let mut expression = values[0].to_string();
        for index in 1..values.len() {
            if (values[index] != 0) && (values[index - 1] % values[index] == 0) {
                expression += match rng.gen_range(0..10) {
                    x if x < 1 => "+",
                    x if x < 2 => "-",
                    x if x < 3 => "*",
                    x if x < 10 => "/",
                    _ => "^", // :^)
                };
            } else {
                expression += match rng.gen_range(0..9) {
                    x if x < 3 => "+",
                    x if x < 6 => "-",
                    x if x < 9 => "*",
                    _ => "^", // :^)
                };
            }

            expression += &values[index].to_string();
        }

        // generating an expression
        println!("{expression}");
        let answer = match simexer(&expression) {
            Ok(value) => value,
            Err(_) => {
                println!("couldn't parse {expression}");
                return generate_puzzle(config);
            }
        };
        prompt += "= ";
        prompt += &answer.to_string();
        Puzzle {
            answer,
            prompt,
            expression,
        }
    }

    #[derive(PartialEq, Ord, PartialOrd, Eq, Clone)]
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

        let mut parse_operator = |opr: Operators,
                                  opn: fn(i64, i64) -> i64,
                                  cond: fn(i64, i64) -> bool|
         -> Result<(), ()> {
            while ops.contains(&opr) {
                let index: usize = ops.clone().into_iter().position(|x| x == opr).unwrap();
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
        parse_operator(Operators::Div, |x, y| x / y, |x, y| x % y == 0)?;
        parse_operator(Operators::Mul, |x, y| x * y, |_x, _y| true)?;
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

    pub fn deparenthesizer(input: &String) -> Result<i64, &str> {
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
}
