use crate::app_error::AppError;
use crate::config::Config;
use crate::gpt::QuestionParams;
use shared::locale::Language;


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

    pub fn set_target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }



    pub fn set_question(mut self, question: &str) -> Self {
        if let Ok(q) = shared::gpt::sanitize_question(&question.to_string()) {
            self.question = Some(q);
            self.original_question = Some(question.to_string());
            return self;
        }
        self
    }

    pub fn set_language(mut self, language: &Language) -> Self {
        self.language = Some(language.clone());
        self
    }

    pub fn create(self) -> Result<Self, AppError> {
        if self.target.is_none() || self.question.is_none() || self.language.is_none() {
            return Err(AppError::InvalidInput);
        }
        Ok(self)
    }

    pub fn get_original_question(&self) -> String {
        self.original_question.clone().unwrap()
    }

    pub fn build_question(&self) -> String {
        self.question.clone().unwrap()
    }
}
