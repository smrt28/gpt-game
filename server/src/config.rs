#![allow(dead_code)]
use std::path::*;
use serde::Deserialize;
use std::fs;
use log::error;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub debug: bool,
    pub www: Www,
    pub gpt: Gpt,
    pub logfile: Option<String>,
    pub dirs: Dirs,
}


pub enum DirType {
    Www,
    Assets,
    Dist,
    Root,
}


#[derive(Deserialize, Debug, Clone)]
pub struct Dirs {
    pub pkg: Option<String>,
    pub pkg_www: Option<String>,
    pub pkg_dist: Option<String>,
    pub pkg_assets: Option<String>,
}


impl Dirs {
    pub fn get_path(&self, dir_type: DirType) -> PathBuf {
        PathBuilder::new()
            .join_opt(self.pkg.as_ref())
            .join_opt(match dir_type {
                DirType::Www => self.pkg_www.as_ref(),
                DirType::Assets => self.pkg_assets.as_ref(),
                DirType::Dist => self.pkg_dist.as_ref(),
                DirType::Root => None,
            }).build()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Www {
    pub port: u16,
}


#[derive(Deserialize, Debug, Clone)]
pub struct Gpt {
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

    fn join_opt(mut self, path: Option<&String>) -> Self {
        if let Some(p) = path {
            self.0 = self.0.join(p);
        }
        self
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

    pub fn get_identities_file(&self, lang: &shared::locale::Language) -> PathBuf {
        let filename = match lang {
            shared::locale::Language::English => "identities_en.txt",
            shared::locale::Language::Czech => "identities_cs.txt",
        };
        self.dirs.get_path(DirType::Assets).join(filename)
    }

    fn get_path(&self, component: &str, file: &str) -> PathBuf {
        PathBuilder::new()
            .join(component)
            .join(file)
            .build()
    }

    fn read_path(&self, dir: DirType, file: &str) -> Result<String, anyhow::Error> {
        let f = self.dirs.get_path(dir).join(file);
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

        c.gpt.gpt_instructions = c.read_path(DirType::Assets, "instructions.txt")?;
        c.gpt.gpt_key = c.read_path(DirType::Root, &c.gpt.key_file)?.trim().to_string();
        Ok(c)
    }
}
