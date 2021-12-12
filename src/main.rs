use io::prelude::*;
use std::io;

use rs_lambda::{Lexer, Parser};

fn main() -> io::Result<()> {
    match {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let tokens = Lexer::new(&buffer);
        let mut parser = Parser::new(tokens);
        parser.parse()
    } {
        Ok(ast) => println!("{:#?}", ast),
        Err(err) => println!("Error = {:?}", err),
    };
    Ok(())
}
