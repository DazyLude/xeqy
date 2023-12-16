use dialoguer::Input;

use xeqy::you_know_the_rules::{deparenthesizer, generate_puzzle};
use xeqy::ConfigData;
const HELP_MESSAGE: &str = r#"
This game is called x=y. It's goal is to solve a randomly generated puzzle.
You need to rearrange an x number of digits, using basic arithmetic operations and parentheses, to make an expression that equals to y.
For example, for a given prompt "1 2 3 4 = 10", all of "1+2+3+4", "3*4-1*2" and "(2*3+4)^1" are correct answers.
Type settings to change settings of the game;
"#;

fn main() {
    // loading config
    let mut config = ConfigData::load_config();
    println!("This is Xeqy!\nType help to display help message, or type start to begin the game!");

    let mut run = true;

    while run {
        let input = Input::<String>::new().interact().unwrap();
        match input.trim().to_ascii_lowercase().as_str() {
            // "main menu"
            "exit" => run = false,
            "help" => println!("{}", HELP_MESSAGE),
            "stats" => println!("Your current score is: {}", config.score),
            "start" => start_game(&mut config),
            "settings" => config_editor(&mut config),

            _ => println!("I can't understand your input."),
        }
    }

    // generating a config file with a new score
    config.save_config().unwrap();
    println!("Bye!");
}

fn start_game(config: &mut ConfigData) -> () {
    let puzz = generate_puzzle(config.x);
    let input = Input::<String>::new().interact().unwrap();
    match input.trim().to_ascii_lowercase().as_str() {
        "give up" => println!("Alright. The answer was: {}", puzz.expression),
        _ => match deparenthesizer(&input.to_string()) {
            Ok(value) => {
                if value == puzz.answer {
                    println!("victory!");
                    config.score += config.x as i64;
                } else {
                    println!("Your expression has a vaule of {value}");
                }
            }
            Err(_) => todo!(),
        },
    }
}

fn config_editor(config: &mut ConfigData) -> () {
    print!(
        "You've entered configuration editing mode.\n
To change a setting, enter it's name, then enter new value.\n
Current settings:
x:\t\t{}\t(amount of digits in the puzzle)\n
max_size:\t{}\t(maximum amount of digits in a generated number within an expression",
        config.x, config.max_size
    );
    let input2 = Input::<u32>::new().interact().unwrap();
    config.x = input2;
}
