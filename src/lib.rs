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

pub mod confmap;

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
        confmap::add_config_path(path_str);
        confmap::set_config_name("config.json");
        confmap::read_config();
        if path.as_path().exists() {
            std::fs::remove_file(path.as_path()).expect("failed to delete test file");
        }
        assert_eq!(Some("YesMan".to_string()), confmap::get_string("testGetString"));
        assert_eq!(Some(43), confmap::get_int64("testGetInt64"));
        assert_eq!(Some(vec!["+44 1234567".to_string(), "+44 2345678".to_string()]), confmap::get_string_array("testGetStringArray"));
    }
}
