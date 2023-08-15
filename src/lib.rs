//! # confmap
//!
//! A library for reading config file into a map in memory.
//! This library is based on serde_json and once_cell.
//! after the config file is read, you can easily get the config by using get_string, get_int64, get_bool...
//! This library is created because I cannot find a library like this in rust. (the idea is the same to viper package in golang)
//!
//! example:
//! put a json format file in your project folder like this:
//!
//!         config.json
//!         {
//!             "testGetString": "YesMan",
//!             "testGetInt64": 43,
//!             "testGetStringArray": [
//!                 "+44 1234567",
//!                 "+44 2345678"
//!             ]
//!         }
//!
//! add dependency in Cargo.toml:
//!
//!     [dependencies]
//!
//!     confmap = "1.0.0"
//!
//! in your project main.rs:
//!
//!     use confmap;
//!
//!     fn main() {
//!
//!         confmap::add_config_path(path_str);
//!
//!         confmap::set_config_name("config.json");
//!
//!         confmap::read_config();
//!
//!         assert_eq!(Some("YesMan".to_string()), confmap::get_string("testGetString"));
//!
//!         assert_eq!(Some(43), confmap::get_int64("testGetInt64"));
//!
//!         assert_eq!(Some(vec!["+44 1234567".to_string(), "+44 2345678".to_string()]), confmap::get_string_array("testGetStringArray"));
//!
//!     }

use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use serde_json::{Map, Value};

struct ConfigSerde;

static mut CONFIG_NAME: String = String::new();
static mut CONFIG_PATH: String = String::new();
static CONFIGS: Lazy<Arc<Mutex<Map<String, Value>>>> = Lazy::new(|| {
    let m = Map::new();
    Arc::new(Mutex::new(m))
});

impl ConfigSerde {
    fn parse_value(value_ref: &Value) -> Value {
        value_ref.clone()
    }

    fn read_config(config_path: &str) -> Result<Map<String, Value>, Box<dyn Error>> {
        println!("reading file {}", config_path);
        let config = fs::read_to_string(config_path)?;
        let parsed: Map<String, Value> = serde_json::from_str(config.as_str())?;
        let result = parsed
            .into_iter()
            .map(|(k, v)| (k, ConfigSerde::parse_value(&v)))
            .collect();
        Ok(result)
    }
}

/// Set filename.
/// put config file in the folder of the executable file
/// # Example
/// confmap::set_config_name("config.json");
/// ```
///
pub fn set_config_name(config_name: &str) {
    unsafe { CONFIG_NAME = config_name.to_string(); }
}

/// Add path of the file.
/// this will allow you to put config file in other path
/// # Example
/// confmap::add_config_path("config.json");
/// ```
pub fn add_config_path(path: &str) {
    unsafe {
        #[cfg(target_family = "unix")]
        if path.ends_with("/") {
            CONFIG_PATH = path.to_string();
        } else {
            CONFIG_PATH = path.to_string() + "/";
        }
        #[cfg(target_family = "windows")]
        if path.ends_with("\\") {
            CONFIG_PATH = path.to_string();
        } else {
            CONFIG_PATH = path.to_string() + "\\";
        }
    }
}

/// this function read config file after file path and file name are given.
/// you can use get_string, get_int64 ...etc, to get the value after config file is loaded by this function.
/// # Example
/// ```
/// confmap::read_config();
/// ```
pub fn read_config() {
    if !unsafe { CONFIG_NAME.is_empty() } {
        let path_buf = env::current_exe().expect("Failed to get executable path");
        let paths = fs::read_dir(path_buf.parent().unwrap()).unwrap();
        let mut is_found:bool;
        unsafe{
            let file_path = CONFIG_PATH.to_owned() + &CONFIG_NAME;
            let path = Path::new(&file_path);
            is_found = path.exists() && path.is_file();
        }
        if !is_found {
            for path in paths {
                let path_str = path.unwrap().path();
                let filename = path_str.file_name().unwrap().to_string_lossy();
                unsafe {
                    if filename == CONFIG_NAME.to_string() {
                        #[cfg(target_family = "unix")]
                        {
                            CONFIG_PATH = path_str.clone().parent().unwrap().to_string_lossy().to_string() + "/";
                        }
                        #[cfg(target_family = "windows")]
                        {
                            CONFIG_PATH = path_str.clone().parent().unwrap().to_string_lossy().parse().unwrap() + "\\";
                        }
                        // CONFIG_NAME = filename.parse().unwrap();
                        println!("file is found!!");
                        is_found = true;
                        break;
                    } // else {
                    //     println!("Got: {}, CONFIG_NAME: {:?}", filename, CONFIG_NAME.to_string());
                    // }
                }
            }
        }

        if is_found {
            init_lazy_configs(&mut CONFIGS.lock().unwrap());
        } else {
            println!("file is not found");
        }
    }
}

fn init_lazy_configs(input: &mut Map<String, Value>) {
    let path = unsafe { CONFIG_PATH.to_string() + &CONFIG_NAME };
    println!("init_lazy_configs path: {}", path);
    match ConfigSerde::read_config(&path) {
        Ok(configs) => {
            for (k, v) in configs.iter() {
                input.insert(k.clone(), v.clone()); // Assuming Value is Cloneable
            }
        }
        Err(_e) => {
            // not thing to do
        }
    }
    println!("configs: {:?}", input);
}

/// this function will return Option<String> when you put a key argument.
/// # Example
/// ```
/// confmap::get_string("testGetString");
/// ```
pub fn get_string(key: &str) -> Option<String> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        value.as_str().map(|s| s.to_string())
    } else {
        None
    }
}

/// this function will return Option<Vec<String>> when you put a key argument.
/// # Example
/// ```
/// confmap::get_string_array("testGetStringArray");
/// ```
pub fn get_string_array(key: &str) -> Option<Vec<String>> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        if let Value::Array(arr) = value {
            let mut string_array = Vec::new();
            for element in arr {
                if let Value::String(s) = element {
                    string_array.push(s.clone());
                }
            }
            Some(string_array)
        } else {
            None
        }
    } else {
        None
    }
}

/// this function will return Option<i64> when you put a key argument.
/// # Example
/// ```
/// confmap::get_int64("testGetInt64");
/// ```
pub fn get_int64(key: &str) -> Option<i64> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        match value {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    } else {
        None
    }
}

/// this function will return Option<Vec<i64>> when you put a key argument.
/// # Example
/// ```
/// confmap::get_int64_array("testGetFloat64Array");
/// ```
pub fn get_int64_array(key: &str) -> Option<Vec<i64>> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        if let Value::Array(arr) = value {
            let mut int64_array = Vec::new();
            for element in arr {
                if let Value::Number(n) = element {
                    if let Some(int_value) = n.as_i64() {
                        int64_array.push(int_value);
                    }
                }
            }
            Some(int64_array)
        } else {
            None
        }
    } else {
        None
    }
}

/// this function will return Option<i32> when you put a key argument.
/// # Example
/// ```
/// confmap::get_int32("testGetInt32");
/// ```
pub fn get_i32(key: &str) -> Option<i32> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        match value {
            Value::Number(n) => n.as_i64().map(|n| n as i32),
            _ => None,
        }
    } else {
        None
    }
}

/// this function will return Option<i16> when you put a key argument.
/// # Example
/// ```
/// confmap::get_int16("testGetInt16");
/// ```
pub fn get_i16(key: &str) -> Option<i16> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        match value {
            Value::Number(n) => n.as_i64().map(|n| n as i16),
            _ => None,
        }
    } else {
        None
    }
}

/// this function will return Option<i8> when you put a key argument.
/// # Example
/// ```
/// confmap::get_int8("testGetInt8");
/// ```
pub fn get_int8(key: &str) -> Option<i8> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        match value {
            Value::Number(n) => n.as_i64().map(|n| n as i8),
            _ => None,
        }
    } else {
        None
    }
}

/// this function will return Option<f64> when you put a key argument.
/// # Example
/// ```
/// confmap::get_float64("testGetFloat64");
/// ```
pub fn get_float64(key: &str) -> Option<f64> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        match value {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    } else {
        None
    }
}

/// this function will return Option<Vec<f64>> when you put a key argument.
/// # Example
/// ```
/// confmap::get_float64_array("testGetFloat64Array");
/// ```
pub fn get_float64_array(key: &str) -> Option<Vec<f64>> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        if let Value::Array(arr) = value {
            let mut float64_array = Vec::new();
            for element in arr {
                if let Value::Number(n) = element {
                    if let Some(int_value) = n.as_f64() {
                        float64_array.push(int_value);
                    }
                }
            }
            Some(float64_array)
        } else {
            None
        }
    } else {
        None
    }
}

/// this function will return Option<f32> when you put a key argument.
/// # Example
/// ```
/// confmap::get_float32("testGetFloat32");
/// ```
pub fn get_float32(key: &str) -> Option<f32> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        match value {
            Value::Number(n) => n.as_f64().map(|n| n as f32),
            _ => None,
        }
    } else {
        None
    }
}

/// this function will return Option<bool> when you put a key argument.
/// # Example
/// ```
/// confmap::get_bool("testGetBool");
/// ```
pub fn get_bool(key: &str) -> Option<bool> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        value.as_bool()
    } else {
        None
    }
}

/// this function will return Option<serde_json::Value> when you put a key argument.
/// # Example
/// ```
/// confmap::get("testGet");
/// ```
pub fn get(key: &str) -> Option<Value> {
    CONFIGS.lock().unwrap().get(key).cloned()
}

/// this function will return Option<Vec<serde_json::Value>> when you put a key argument.
/// # Example
/// ```
/// confmap::get_array("testGetArray");
/// ```
pub fn get_array(key: &str) -> Option<Vec<Value>> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(value) = configs.get(key) {
        if let Value::Array(arr) = value {
            let mut array = Vec::new();
            for element in arr {
                if let Value::Object(_) = element {
                    array.push(element.clone());
                }
            }
            Some(array)
        } else {
            None
        }
    } else {
        None
    }
}

/// this function will return Option<Map<String, Value>> when you put a key argument.
/// # Example
/// ```
/// confmap::get_map("testGetMap");
/// ```
pub fn get_map(key: &str) -> Option<Map<String, Value>> {
    let configs = CONFIGS.lock().unwrap();
    if let Some(map) = configs.get(key) {
        map.as_object().cloned()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::io::Write;
    use std::path::{PathBuf};
    use super::*;

    #[test]
    fn it_works() {
        let data = r#"
        {
            "testGetString": "YesMan",
            "testGetInt64": 43,
            "testGetStringArray": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;
        let path_buf = env::current_exe().expect("Failed to get executable path");
        let path_str = path_buf
            .ancestors()
            .nth(4) // Adjust the number as needed to reach the desired parent directory
            .expect("Invalid number of ancestors")
            .to_str()
            .expect("Failed to convert path to string");
        let mut path = PathBuf::from(path_str);
        path.push("config.json");
        let mut file = std::fs::File::create(path.clone()).expect("create failed");
        file.write_all(data.as_bytes()).expect("write failed");
        add_config_path(path_str);
        set_config_name("config.json");
        read_config();
        if path.as_path().exists() {
            std::fs::remove_file(path.as_path()).expect("failed to delete test file");
        }
        assert_eq!(Some("YesMan".to_string()), get_string("testGetString"));
        assert_eq!(Some(43), get_int64("testGetInt64"));
        assert_eq!(Some(vec!["+44 1234567".to_string(), "+44 2345678".to_string()]), get_string_array("testGetStringArray"));
    }
}
