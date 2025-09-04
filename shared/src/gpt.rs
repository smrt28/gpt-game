use anyhow::Error;

pub fn check_question(question: &str) -> Result<(), Error> {
    if question.len() > 120 {
        return Err(anyhow::anyhow!("Too long question").into());
    }
    let count = question.chars().filter(|c| !c.is_whitespace()).count();
    if count < 5 {
        return Err(anyhow::anyhow!("Is it even a question?").into());
    }
    Ok(())
}

pub fn sanitize_question(question: &str) -> Result<String, Error> {
    check_question(question)?;
    let clean_question = question.replace(['[', ']'], "/");
    Ok(format!("question: [{}]", clean_question))
}