use std::str::FromStr;

use crate::query::token::Token;

pub struct Lexer {
    target_data: String,
}

impl Lexer {
    pub fn new(target_data: impl ToString) -> Self {
        Lexer {
            target_data: target_data.to_string(),
        }
    }

    pub fn exec(&mut self) -> Vec<Token> {
        self.tokenize()
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let mut query = self.target_data.chars().into_iter().clone();
        let mut pretty_tokens = vec![];
        let mut objected_tokens = vec![];
        let mut temp: Vec<char> = vec![];
        let noize_list = [' '];

        loop {
            use std::iter::FromIterator;
            if let Some(c) = query.next() {
                if noize_list.contains(&c) {
                    if !temp.is_empty() {
                        let token = String::from_iter(temp);
                        temp = vec![];
                        pretty_tokens.push(token);
                    }
                } else {
                    match c {
                        ',' | ';' | '(' | ')' => {
                            if !temp.is_empty() {
                                let token = String::from_iter(temp);
                                temp = vec![];
                                pretty_tokens.push(token);
                            }
                            pretty_tokens.push(String::from(c))
                        }
                        '>' | '<' | '=' => {
                            if !temp.is_empty() {
                                let token = String::from_iter(temp);
                                temp = vec![];
                                pretty_tokens.push(token);
                            }
                            temp.push(c);
                            if let Some(c) = query.next() {
                                match c {
                                    '>' | '<' | '=' => {
                                        temp.push(c);
                                        let token = String::from_iter(temp);
                                        temp = vec![];
                                        pretty_tokens.push(token);
                                    }
                                    _ => {
                                        let token = String::from_iter(temp);
                                        temp = vec![];
                                        pretty_tokens.push(token);
                                        temp.push(c);
                                    }
                                }
                            }
                        }
                        _ => temp.push(c),
                    }
                }
            } else {
                if !temp.is_empty() {
                    let token = String::from_iter(temp);
                    temp = vec![];
                    pretty_tokens.push(token);
                }
                break;
            }
        }

        let mut tokens = pretty_tokens.iter();
        if let Some(token) =  tokens.next(){
            match token.to_lowercase().as_str() {
                "select" => objected_tokens.push(Token::SELECT),
                "from" => objected_tokens.push(Token::FROM),
                "insert" => objected_tokens.push(Token::INSERT),
                "into" => objected_tokens.push(Token::INTO),
                "create" => objected_tokens.push(Token::CREATE),
                "table" => objected_tokens.push(Token::TABLE),
                "where" => objected_tokens.push(Token::WHERE),
                "and" => objected_tokens.push(Token::AND),
                "integer" => objected_tokens.push(Token::INTEGER),
                "text" => objected_tokens.push(Token::TEXT),
                "and" | "or" | "not" | "<" | ">"| "=" | ">=" | "<=" => {
                    let previous_token = objected_tokens.pop().unwrap();
                    match token.as_str() {
                        "and" => {}
                        "or" => {}
                        "not" => {}
                        "<" => {}
                        ">" => {}
                        "=" => {} 
                        ">=" => {}
                        "<=" => {}
                        _ => panic!()
                    }
                }
                ";" => objected_tokens.push(Token::EOF),
                _ => objected_tokens.push(Token::Phrase(token.clone()))
            }
        }

        return objected_tokens;
    }
}
