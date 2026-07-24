use std::iter::Peekable;
use std::str::Chars;

/// SQL Token
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    Begin,
    Commit,
    Rollback,
    And,
    Or,
    Not,
    In,
    Like,
    Is,
    Null,
    Primary,
    Key,
    Default,
    NotNull,
    Asc,
    Desc,
    Order,
    By,
    Limit,
    As,

    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),

    // Operators
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Punctuation
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Dot,

    // Special
    Eof,
}

pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        match self.chars.peek() {
            None => Ok(Token::Eof),
            Some(&ch) => match ch {
                '(' => { self.chars.next(); Ok(Token::LeftParen) }
                ')' => { self.chars.next(); Ok(Token::RightParen) }
                ',' => { self.chars.next(); Ok(Token::Comma) }
                ';' => { self.chars.next(); Ok(Token::Semicolon) }
                '.' => { self.chars.next(); Ok(Token::Dot) }
                '*' => { self.chars.next(); Ok(Token::Star) }
                '/' => { self.chars.next(); Ok(Token::Slash) }
                '%' => { self.chars.next(); Ok(Token::Percent) }
                '+' => { self.chars.next(); Ok(Token::Plus) }
                '-' => {
                    self.chars.next();
                    // Check for comment
                    if self.chars.peek() == Some(&'-') {
                        self.skip_comment();
                        self.next_token()
                    } else {
                        Ok(Token::Minus)
                    }
                }
                '=' => { self.chars.next(); Ok(Token::Eq) }
                '!' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        Ok(Token::Ne)
                    } else {
                        Err("Expected '=' after '!'".to_string())
                    }
                }
                '<' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        Ok(Token::Le)
                    } else if self.chars.peek() == Some(&'>') {
                        self.chars.next();
                        Ok(Token::Ne)
                    } else {
                        Ok(Token::Lt)
                    }
                }
                '>' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        Ok(Token::Ge)
                    } else {
                        Ok(Token::Gt)
                    }
                }
                '\'' | '"' => self.read_string(),
                '0'..='9' => self.read_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
                _ => Err(format!("Unexpected character: {}", ch)),
            },
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        // Skip -- comment
        while let Some(&ch) = self.chars.peek() {
            if ch == '\n' {
                break;
            }
            self.chars.next();
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let quote = self.chars.next().unwrap();
        let mut value = String::new();

        loop {
            match self.chars.peek() {
                None => return Err("Unterminated string".to_string()),
                Some(&ch) => {
                    if ch == quote {
                        self.chars.next();
                        return Ok(Token::String(value));
                    } else if ch == '\\' {
                        self.chars.next();
                        match self.chars.next() {
                            Some('n') => value.push('\n'),
                            Some('t') => value.push('\t'),
                            Some('\\') => value.push('\\'),
                            Some(c) if c == quote => value.push(c),
                            Some(c) => {
                                value.push('\\');
                                value.push(c);
                            }
                            None => return Err("Unterminated escape".to_string()),
                        }
                    } else {
                        value.push(ch);
                        self.chars.next();
                    }
                }
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut value = String::new();
        let mut is_float = false;

        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.chars.next();
            } else if ch == '.' && !is_float {
                is_float = true;
                value.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        if is_float {
            value.parse::<f64>()
                .map(Token::Float)
                .map_err(|e| format!("Invalid float: {}", e))
        } else {
            value.parse::<i64>()
                .map(Token::Integer)
                .map_err(|e| format!("Invalid integer: {}", e))
        }
    }

    fn read_identifier(&mut self) -> Result<Token, String> {
        let mut value = String::new();

        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                value.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        // Check for keywords
        match value.to_uppercase().as_str() {
            "SELECT" => Ok(Token::Select),
            "FROM" => Ok(Token::From),
            "WHERE" => Ok(Token::Where),
            "INSERT" => Ok(Token::Insert),
            "INTO" => Ok(Token::Into),
            "VALUES" => Ok(Token::Values),
            "UPDATE" => Ok(Token::Update),
            "SET" => Ok(Token::Set),
            "DELETE" => Ok(Token::Delete),
            "CREATE" => Ok(Token::Create),
            "TABLE" => Ok(Token::Table),
            "BEGIN" => Ok(Token::Begin),
            "COMMIT" => Ok(Token::Commit),
            "ROLLBACK" => Ok(Token::Rollback),
            "AND" => Ok(Token::And),
            "OR" => Ok(Token::Or),
            "NOT" => Ok(Token::Not),
            "IN" => Ok(Token::In),
            "LIKE" => Ok(Token::Like),
            "IS" => Ok(Token::Is),
            "NULL" => Ok(Token::Null),
            "PRIMARY" => Ok(Token::Primary),
            "KEY" => Ok(Token::Key),
            "DEFAULT" => Ok(Token::Default),
            "NOTNULL" | "NOT_NULL" => Ok(Token::NotNull),
            "ASC" => Ok(Token::Asc),
            "DESC" => Ok(Token::Desc),
            "ORDER" => Ok(Token::Order),
            "BY" => Ok(Token::By),
            "LIMIT" => Ok(Token::Limit),
            "AS" => Ok(Token::As),
            "TRUE" => Ok(Token::Boolean(true)),
            "FALSE" => Ok(Token::Boolean(false)),
            _ => Ok(Token::Identifier(value)),
        }
    }
}
