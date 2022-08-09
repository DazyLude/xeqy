use std::path::Path;

const CONFIG_LENGTH: usize = 12;
const CONFIG_PATH: &str = ".config";

// Config stores score and settings
pub struct ConfigData {
    pub x: u32,
    pub score: i64,
}

impl ConfigData {
    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        for byte in self.x.to_be_bytes() {
            data.push(byte)
        }
        for byte in self.score.to_be_bytes() {
            data.push(byte)
        }
        data
    }
    pub fn save_config(cfg: ConfigData) -> Result<(), std::io::Error> {
        std::fs::write(Path::new(CONFIG_PATH), cfg.serialize())?;
        Ok(())
    }

    pub fn deserialize(data: Vec<u8>) -> Self {
        if data.len() != CONFIG_LENGTH {
            println!("trying to deserialize byte sequence of length {}, length of {} was expected. Loading default config.",
                data.len(),
                CONFIG_LENGTH
            );
            return ConfigData { x: 4, score: 0 };
        }

        ConfigData {
            x: u32::from_be_bytes(data[0..4].try_into().unwrap()),
            score: i64::from_be_bytes(data[4..12].try_into().unwrap()),
        }
    }
    pub fn load_config() -> ConfigData {
        let config_path = Path::new(".config");
        if config_path.exists() {
            // loads an existing config
            return ConfigData::deserialize(std::fs::read(config_path).unwrap());
        } else {
            // or loads default values
            return ConfigData { x: 4, score: 0 };
        };
    }
}

pub fn calc_expression(expr: String) -> Result<i64, ()> {
    //this function requires an expression to have only values and operators
    let operators: [&str, 5] = ["^", "*", "/", "+", "-"];
    for operator in operators {
        match expr.find(operator) {
            None => continue;
            Some(pos) => 
        }
    }
}
