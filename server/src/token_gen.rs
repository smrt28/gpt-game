use rand::Rng;
use shared::token::*;

pub trait TokenGen {
    fn new(token_type: TokenType) -> Self;
}

fn random_bytes(leading_letter: u8) -> [u8; TOKEN_LENGTH] {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let mut buf = [0u8; TOKEN_LENGTH];
    buf[0] = leading_letter;
    for b in &mut buf[1..] {
        *b = CHARSET[rng.random_range(0..CHARSET.len())];
    }
    buf
}
impl TokenGen for Token {
    fn new(token_type: TokenType) -> Self {
        Self {
            token: random_bytes(token_type.leading_byte())
        }
    }
}