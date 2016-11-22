use std::io::prelude::*;
use std::fs::File;
use rustc_serialize::json;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Config {
    pub target: String
}

pub fn read(filename: &str) -> Config {
    let mut file = File::open(filename).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    json::decode(&text).unwrap()
}
