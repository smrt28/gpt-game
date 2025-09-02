#![allow(dead_code)]
use std::path::*;
use serde::Deserialize;
use std::fs;
use log::error;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub debug: bool,
    pub root: String,
    pub www: Www,
    pub gpt: Gpt,



}

#[derive(Deserialize, Debug, Clone)]
pub struct Www {
    pub port: u16,
    pub path: String,
    pub dist: String,
}


#[derive(Deserialize, Debug, Clone)]
pub struct Gpt {
    pub path: String,
    pub instructions_file: String,
    pub key_file: String,
    pub max_clients_count: u32,


    #[serde(skip)]
    pub gpt_instructions: String,
    #[serde(skip)]
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

        let f = self.get_path(component, file);
        match fs::read_to_string(f.clone()) {
            Ok(res) => return Ok(res),
            Err(err) => {
                error!("file {} not found, err={}", f.to_str().unwrap(), err);
                return Err(err.into());
            }
        }
    }

    pub fn read(file_name: &str) -> Result<Config, anyhow::Error> {
        let contents = fs::read_to_string(file_name)?;
        let mut c = toml::from_str::<Config>(&contents)?;

        c.gpt.gpt_instructions = c.read_path(&c.gpt.path, &c.gpt.instructions_file)?;
        c.gpt.gpt_key = c.read_path(&c.gpt.path, &c.gpt.key_file)?.trim().to_string();
        Ok(c)
    }
}
