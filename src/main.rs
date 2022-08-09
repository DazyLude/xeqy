use dialoguer::Input;
use rand::{thread_rng, Rng};
use xeqy::ConfigData;

const HELP_MESSAGE: &str = r#"
This game is called x=y. It's goal is to solve a randomly generated puzzle.
You need to rearrange an x number of digits, using basic arithmetic operations and parentheses, to make an expression that equals to y.
For example, for a given prompt "1 2 3 4 = 10", all of "1+2+3+4", "1*2+3+4" and "3*4-1*2" are correct answers.
"#;

fn main() {
    // loading config
    let config = ConfigData::load_config();

    println!("This is Xeqy!\nType help to display help message, or type start to begin the game!");

    let mut run = true;
    let mut waiting_for_an_answer = false;
    while run {
        let input = Input::<String>::new().interact_text().unwrap();
        match input.as_str() {
            "exit" => run = false,
            "help" => println!("{}", HELP_MESSAGE),
            "start" => {
                println!("Your current score is: {}", config.score);
                let puzz = generate_puzzle(config.x);
                println!("{}", puzz.prompt);
                waiting_for_an_answer = true;
            }
            _ => {
                if waiting_for_an_answer {
                    match calc_string(input) {
                        Ok(value) => {}
                        Err(_) => println!("I can't understand your input, try again!"),
                    }
                } else {
                    println!("Unknown command. Try again!")
                }
            }
        }
    }

    // generating a config file with a new score
    save_config().unwrap();
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
        max_size = if (x / 2) < (x - size) {
            x / 2
        } else {
            x - size
        };
        let add_size = rng.gen_range(1..=max_size);
        if size == 1 {
            values.push(rng.gen_range(0..10));
        } else {
            values.push(rng.gen_range(10_u32.pow(size - 1)..10_u32.pow(size)).into());
        }
        size += add_size;
    }
    // generating an expression
    let answer = 0;
    let prompt = "ass".to_string();
    Puzzle { answer, prompt }
}

// returns ok(value of an expression) if string is "calculable", or an error unit
// basically a parser
// idea is to divide expression into a set of subexpressions and calculate then one by one

const OPERATIONS: [&'static str; 6] = ["^", "pow", "*", "/", "+", "-"];

fn calc_string(input: String) -> Result<i64, ()> {
    // a simple expression, which contains only operators from OPERATIONS array
    fn calc_expression(mut expr: String) -> Result<i64, ()> {
        expr = expr.trim().to_string();
        for operator in OPERATIONS {
            if expr.contains(operator) {
                let position = expr.find(operator).unwrap();
                let right_side = expr[position + 1..].trim().chars();
                let mut right_num_length = 0;
            }
        }

        match expr.parse::<i64>() {
            Ok(val) => return Ok(val),
            Err(_) => return Err(()),
        }
    }

    // parentheses divide expression into subexpressions, which then are calculated.
    fn get_subexpression(mut expr: String) -> Result<i64, ()> {
        if expr.contains("(") {
            let left_brace = expr.find("(").unwrap();
            let right_brace = match expr.rfind(")") {
                None => return Err(()),
                Some(val) => val,
            };
            let subexpression_value =
                match get_subexpression(String::from(&expr[left_brace + 1..right_brace])) {
                    Ok(val) => val,
                    Err(_) => return Err(()),
                };
            let left_side = String::from(&expr[..left_brace]);
            let right_side = &String::from(&expr[right_brace + 1..]);
            expr = left_side + &subexpression_value.to_string() + right_side;
        }
        calc_expression(expr)
    }

    get_subexpression(input)
}
