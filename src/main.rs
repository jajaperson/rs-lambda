use std::io;
use std::io::prelude::*;

fn main() {
    match {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        let tokens = Lexer::new(&buffer);
        let mut parser = Parser::new(tokens);
        parser.parse()
    } {
        Ok(ast) => println!("{:#?}", ast),
        Err(err) => println!("Error = {:?}", err),
    };
}

#[derive(Debug)]
enum Token {
    LParen,
    RParen,
    Lambda,
    Dot,
    Identifier(String),
    Eof,
}

use std::iter::Peekable;
use std::str::Chars;

struct Lexer<'a> {
    chars_peekable: Peekable<Chars<'a>>,
    buffer: String,
    index: isize,
}

impl<'a> Lexer<'a> {
    fn new<'b>(code: &'b str) -> Lexer<'b> {
        Lexer {
            chars_peekable: code.chars().peekable(),
            buffer: String::new(),
            index: -1,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.index += 1;
        return loop {
            match self.chars_peekable.next() {
                None => break None,
                Some(ch) => match ch {
                    '(' => break Some(Token::LParen),
                    ')' => break Some(Token::RParen),
                    'λ' | '\\' => break Some(Token::Lambda),
                    '.' => break Some(Token::Dot),
                    '\0' => break Some(Token::Eof),
                    _ if ch.is_alphanumeric() => {
                        self.buffer.push(ch);
                        match self.chars_peekable.peek() {
                            Some(nch) if nch.is_alphanumeric() && nch != &'λ' => (),
                            _ => {
                                let id_str = self.buffer.clone();
                                self.buffer.clear();
                                break Some(Token::Identifier(id_str));
                            }
                        }
                    }
                    _ => (),
                },
            }
        };
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.chars_peekable.size_hint().1)
    }
}

#[derive(Debug)]
enum LambdaTerm {
    Abstraction {
        bound_variable: String,
        return_term: Box<LambdaTerm>,
    },
    Application {
        function: Box<LambdaTerm>,
        argument: Box<LambdaTerm>,
    },
    Variable(String),
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    paren_index: isize,
}

#[derive(Debug)]
enum ParserError {
    PrematureEnd,
    ParenOutOfBounds {
        paren_index_bound: isize,
        paren_index: isize,
    },
    ExpectedIdentifierGot(Token),
    ExpectedGot(Token, Token),
    Unexpected(Token),
    UnmatchedParens(isize),
}

impl<'a> Parser<'a> {
    fn new<'b>(lexer: Lexer<'b>) -> Parser<'b> {
        Parser {
            lexer,
            paren_index: 0,
        }
    }

    fn parse(&mut self) -> Result<LambdaTerm, ParserError> {
        let root_term = self.parse_term(self.paren_index)?;
        if self.paren_index != 0 {
            Err(ParserError::UnmatchedParens(self.paren_index))
        } else {
            Ok(root_term)
        }
    }

    fn parse_term(&mut self, paren_index_bound: isize) -> Result<LambdaTerm, ParserError> {
        self.check_bounds(paren_index_bound)?;
        let mut term = match self.lexer.next() {
            Some(token) => match token {
                Token::Lambda => self.parse_abstraction(self.paren_index),
                Token::Dot => Err(ParserError::Unexpected(Token::Dot)),
                Token::RParen => Err(ParserError::Unexpected(Token::RParen)),
                Token::LParen => {
                    self.paren_index += 1;
                    self.parse_term(self.paren_index)
                }
                Token::Identifier(id) => Ok(LambdaTerm::Variable(id)),
                Token::Eof => Err(ParserError::PrematureEnd),
            },
            None => Err(ParserError::PrematureEnd),
        }?;
        while self.paren_index >= paren_index_bound {
            match self.lexer.next() {
                Some(token) => match token {
                    Token::LParen => {
                        self.paren_index += 1;
                        term = LambdaTerm::Application {
                            function: Box::new(term),
                            argument: Box::new(self.parse_term(self.paren_index)?),
                        };
                    }
                    Token::RParen => {
                        self.paren_index -= 1;
                    }
                    Token::Lambda => {
                        term = LambdaTerm::Application {
                            function: Box::new(term),
                            argument: Box::new(self.parse_abstraction(self.paren_index)?),
                        }
                    }
                    Token::Identifier(id) => {
                        term = LambdaTerm::Application {
                            function: Box::new(term),
                            argument: Box::new(LambdaTerm::Variable(id)),
                        }
                    }
                    Token::Eof => (),
                    Token::Dot => Err(ParserError::Unexpected(Token::Dot))?,
                },
                None => break,
            }
        }
        Ok(term)
    }

    fn parse_abstraction(&mut self, paren_index_bound: isize) -> Result<LambdaTerm, ParserError> {
        self.check_bounds(paren_index_bound)?;
        match self.lexer.next() {
            Some(expected_identifier) => match expected_identifier {
                Token::Identifier(bound_variable) => match self.lexer.next() {
                    Some(expected_dot) => match expected_dot {
                        Token::Dot => Ok(LambdaTerm::Abstraction {
                            bound_variable,
                            return_term: Box::new(self.parse_term(self.paren_index)?),
                        }),
                        _ => Err(ParserError::ExpectedGot(Token::Dot, expected_dot)),
                    },
                    None => Err(ParserError::PrematureEnd),
                },
                _ => Err(ParserError::ExpectedIdentifierGot(expected_identifier)),
            },
            None => Err(ParserError::PrematureEnd),
        }
    }

    fn check_bounds(&self, paren_index_bound: isize) -> Result<(), ParserError> {
        if self.paren_index < paren_index_bound {
            Err(ParserError::ParenOutOfBounds {
                paren_index: self.paren_index,
                paren_index_bound,
            })
        } else {
            Ok(())
        }
    }
}
