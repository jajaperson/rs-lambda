#[derive(Debug)]
pub enum Token {
    LParen,
    RParen,
    Lambda,
    Dot,
    Identifier(String),
    Eof,
}

use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars_peekable: Peekable<Chars<'a>>,
    buffer: String,
}

impl<'a> Lexer<'a> {
    pub fn new<'b>(code: &'b str) -> Lexer<'b> {
        Lexer {
            chars_peekable: code.chars().peekable(),
            buffer: String::new(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        return loop {
            match self.chars_peekable.next() {
                None => break None,
                Some(ch) => match ch {
                    '(' => break Some(Token::LParen),
                    ')' => break Some(Token::RParen),
                    'λ' | '\\' => break Some(Token::Lambda),
                    '.' => break Some(Token::Dot),
                    '\0' => break Some(Token::Eof),
                    _ if ch.is_alphanumeric() || ch == '_' => {
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

impl<'a> std::iter::FusedIterator for Lexer<'a> {}

#[derive(Debug)]
pub enum LambdaTerm {
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

use std::collections::HashSet;

impl LambdaTerm {
    pub fn free_variables(&self) -> HashSet<String> {
        fn free_variables_mut(term: &LambdaTerm, set: &mut HashSet<String>) {
            match term {
                LambdaTerm::Variable(id) => {
                    set.insert(id.clone());
                }
                LambdaTerm::Application { function, argument } => {
                    free_variables_mut(function, set);
                    free_variables_mut(argument, set);
                }
                LambdaTerm::Abstraction {
                    bound_variable,
                    return_term,
                } => {
                    free_variables_mut(return_term, set);
                    set.remove(bound_variable);
                }
            }
        }
        let mut set = HashSet::new();
        free_variables_mut(self, &mut set);
        set
    }

    pub fn bound_variables(&self) -> HashSet<String> {
        fn bound_variables_mut(term: &LambdaTerm, set: &mut HashSet<String>) {
            match term {
                LambdaTerm::Variable(_) => (),
                LambdaTerm::Application { function, argument } => {
                    bound_variables_mut(function, set);
                    bound_variables_mut(argument, set);
                }
                LambdaTerm::Abstraction {
                    bound_variable,
                    return_term,
                } => {
                    bound_variables_mut(return_term, set);
                    set.insert(bound_variable.clone());
                }
            }
        }
        let mut set = HashSet::new();
        bound_variables_mut(self, &mut set);
        set
    }
}

use std::fmt;

impl fmt::Display for LambdaTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LambdaTerm::Variable(id) => write!(f, "{}", id)?,
            LambdaTerm::Application { function, argument } => {
                match **function {
                    LambdaTerm::Abstraction { .. } => write!(f, "({}) ", function)?,
                    _ => write!(f, "{} ", function)?,
                }
                match **argument {
                    LambdaTerm::Variable(_) => write!(f, "{}", argument)?,
                    _ => write!(f, "({})", argument)?,
                }
            }
            LambdaTerm::Abstraction {
                bound_variable,
                return_term,
            } => write!(f, "λ{}. {}", bound_variable, return_term)?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ParserError {
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

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    paren_index: isize,
}

impl<'a> Parser<'a> {
    pub fn new<'b>(lexer: Lexer<'b>) -> Parser<'b> {
        Parser {
            lexer,
            paren_index: 0,
        }
    }

    pub fn parse(&mut self) -> Result<LambdaTerm, ParserError> {
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

pub enum DBTerm {
    Variable(usize),
    Application {
        function: Box<DBTerm>,
        argument: Box<DBTerm>,
    },
    Abstraction(Box<DBTerm>),
    FreeVariable(String),
}

pub struct DBLevels(pub DBTerm);

pub struct DBIndices(pub DBTerm);

impl DBTerm {
    pub fn free_variables(&self) -> HashSet<String> {
        fn free_variables_mut(term: &DBTerm, set: &mut HashSet<String>) {
            match term {
                DBTerm::FreeVariable(id) => {
                    set.insert(id.clone());
                }
                DBTerm::Variable(_) => (),
                DBTerm::Abstraction(return_term) => free_variables_mut(return_term, set),
                DBTerm::Application { function, argument } => {
                    free_variables_mut(function, set);
                    free_variables_mut(argument, set);
                }
            }
        }
        let mut set = HashSet::new();
        free_variables_mut(self, &mut set);
        set
    }
}

impl fmt::Display for DBTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBTerm::Variable(id) => write!(f, "{}", id)?,
            DBTerm::FreeVariable(id) => write!(f, "{}", id)?,
            DBTerm::Application { function, argument } => {
                match **function {
                    DBTerm::Abstraction { .. } => write!(f, "({}) ", function)?,
                    _ => write!(f, "{} ", function)?,
                }
                match **argument {
                    DBTerm::Variable(_) | DBTerm::FreeVariable(_) => write!(f, "{}", argument)?,
                    _ => write!(f, "({})", argument)?,
                }
            }
            DBTerm::Abstraction(return_term) => write!(f, "λ {}", return_term)?,
        }
        Ok(())
    }
}

impl From<DBLevels> for DBIndices {
    fn from(levels: DBLevels) -> DBIndices {
        fn reindex(term: DBTerm, abstraction_depth: usize) -> DBTerm {
            match term {
                DBTerm::FreeVariable(id) => DBTerm::FreeVariable(id),
                DBTerm::Variable(level) => DBTerm::Variable(abstraction_depth - level + 1),
                DBTerm::Application { function, argument } => DBTerm::Application {
                    function: Box::new(reindex(*function, abstraction_depth)),
                    argument: Box::new(reindex(*argument, abstraction_depth)),
                },
                DBTerm::Abstraction(return_term) => {
                    DBTerm::Abstraction(Box::new(reindex(*return_term, abstraction_depth + 1)))
                }
            }
        }
        let DBLevels(term) = levels;
        DBIndices(reindex(term, 0))
    }
}

impl From<DBIndices> for DBLevels {
    fn from(indices: DBIndices) -> DBLevels {
        fn reindex(term: DBTerm, abstraction_depth: usize) -> DBTerm {
            match term {
                DBTerm::FreeVariable(id) => DBTerm::FreeVariable(id),
                DBTerm::Variable(index) => DBTerm::Variable(abstraction_depth - index + 1),
                DBTerm::Application { function, argument } => DBTerm::Application {
                    function: Box::new(reindex(*function, abstraction_depth)),
                    argument: Box::new(reindex(*argument, abstraction_depth)),
                },
                DBTerm::Abstraction(return_term) => {
                    DBTerm::Abstraction(Box::new(reindex(*return_term, abstraction_depth + 1)))
                }
            }
        }
        let DBIndices(term) = indices;
        DBLevels(reindex(term, 0))
    }
}

impl fmt::Display for DBLevels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DBLevels(term) = self;
        term.fmt(f)
    }
}

impl fmt::Display for DBIndices {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DBIndices(term) = self;
        term.fmt(f)
    }
}

use std::collections::HashMap;

impl From<LambdaTerm> for DBLevels {
    fn from(lambda: LambdaTerm) -> DBLevels {
        fn convert(
            term: LambdaTerm,
            abstraction_depth: usize,
            level_map: &mut HashMap<String, usize>,
        ) -> DBTerm {
            match term {
                LambdaTerm::Abstraction {
                    bound_variable,
                    return_term,
                } => {
                    let new_depth = abstraction_depth + 1;
                    let opt_old = level_map.insert(bound_variable.clone(), new_depth);
                    let converted_return = convert(*return_term, new_depth, level_map);
                    if let Some(old) = opt_old {
                        level_map.insert(bound_variable, old);
                    } else {
                        level_map.remove(&bound_variable);
                    }
                    DBTerm::Abstraction(Box::new(converted_return))
                }
                LambdaTerm::Application { function, argument } => DBTerm::Application {
                    function: Box::new(convert(*function, abstraction_depth, level_map)),
                    argument: Box::new(convert(*argument, abstraction_depth, level_map)),
                },
                LambdaTerm::Variable(id) => {
                    if let Some(level) = level_map.get(&id) {
                        DBTerm::Variable(*level)
                    } else {
                        DBTerm::FreeVariable(id)
                    }
                }
            }
        }
        let mut level_map = HashMap::new();
        DBLevels(convert(lambda, 0, &mut level_map))
    }
}

impl From<LambdaTerm> for DBIndices {
    fn from(lambda: LambdaTerm) -> DBIndices {
        DBLevels::from(lambda).into()
    }
}
