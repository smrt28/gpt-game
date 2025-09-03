use std::fmt::format;
use tower::util::Either;
use crate::app_error::AppError;
use crate::gpt::QuestionParams;
use crate::config::Config;
use shared::locale::Language;
use crate::locale;


#[derive(Debug, Clone)]
pub struct GameStepBuilder {
    original_question: Option<String>,
    question: Option<String>,
    target: Option<String>,
    language: Option<Language>,
}


impl GameStepBuilder {
    pub fn build_params(&self, config: &Config) -> QuestionParams {
        let mut params = QuestionParams::default();
        let target = self.target.clone().unwrap();
        let language = self.language.clone().unwrap();


        let instructions =
            config.gpt.gpt_instructions
                .replace("{target}", &target.as_str())
                .replace("{language}", language.to_instruction());


        params.set_instructions(instructions);
        params
    }

    pub fn new(_config: &Config) -> Self {
        Self {
            original_question: None,
            question: None,
            target: None,
            language: None,
        }
    }

    pub fn get_target(&self) -> String {
        self.target.clone().unwrap()
    }

    pub fn set_target(&mut self, target: &str) -> &mut Self {
        self.target = Some(target.to_string());
        self
    }

    pub fn sanitize_question(&self, question: &String) -> Result<String, AppError> {
        if question.len() > 120 {
            return Err(anyhow::anyhow!("Too long questuin").into());
        }
        let clean_question = question.replace(['[', ']'], "/");
        Ok(format!("question: [{}]", clean_question))
    }

    pub fn set_question(&mut self, question: &str) -> &mut Self {
        if let Ok(q) = self.sanitize_question(&question.to_string()) {
            self.question = Some(q);
            self.original_question = Some(question.to_string());
            return self;
        }
        self
    }

    pub fn set_language(&mut self, language: &Language) -> &mut Self {
        self.language = Some(language.clone());
        self
    }

    pub fn check(&self) -> Result<(), AppError> {
        if self.target.is_none() || self.question.is_none() || self.language.is_none() {
            return Err(AppError::InvalidInput);
        }
        Ok(())
    }

    pub fn get_original_question(&self) -> String {
        self.original_question.clone().unwrap()
    }

    pub fn build_question(&self) -> String {
        self.question.clone().unwrap()
    }
}