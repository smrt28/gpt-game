#![allow(dead_code)]
use std::path::*;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub debug: bool,
    pub root: String,
    pub www: Www,
    pub gpt: Gpt,

    #[serde(skip)]
    pub data: Data,
}

#[derive(Deserialize, Debug)]
pub struct Www {
    pub port: u16,
    pub path: String,
}


#[derive(Deserialize, Debug)]
pub struct Gpt {
    pub path: String,
    pub instructions_file: String,
    pub key_file: String,
    pub max_clients_count: u32,
}

#[derive(Default, Debug)]
pub struct Data {
    pub gpt_instructions: String,
    pub gpt_key: String,
}

#[derive(Default)]
struct PathBuilder(PathBuf);

impl PathBuilder {
    fn new() -> PathBuilder {
        PathBuilder::default()
    }

    fn join(mut self, path: &str) -> Self {
        if path.starts_with('/') {
            self.0 = PathBuf::new();            
        }
        self.0.push(path);
        self
    }

    fn build(self) -> PathBuf {
        self.0
    }
}

impl Config {
    fn get_path(&self, component: &str, file: &str) -> PathBuf {
        PathBuilder::new()
            .join(&self.root)
            .join(component)
            .join(file)
            .build()
    }

    fn read_path(&self, component: &str, file: &str) -> Result<String, anyhow::Error> {
        Ok(fs::read_to_string(self.get_path(component, file))?)
    }

    pub fn read(file_name: &str) -> Result<Config, anyhow::Error> {
        let contents = fs::read_to_string(file_name)?;
        let mut c = toml::from_str::<Config>(&contents)?;

        c.data.gpt_instructions = c.read_path(&c.gpt.path, &c.gpt.instructions_file)?;
        c.data.gpt_key = c.read_path(&c.gpt.path, &c.gpt.key_file)?;
        Ok(c)
    }
}
