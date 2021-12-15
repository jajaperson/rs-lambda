use io::prelude::*;
use std::io;

use rs_lambda::*;

fn main() -> io::Result<()> {
    match {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let tokens = Lexer::new(&buffer);
        let mut parser = Parser::new(tokens);
        parser.parse()
    } {
        Ok(ast) => {
            println!("Free Variables: {:#?}", ast.free_variables());
            println!("Bound Variables: {:#?}", ast.bound_variables());
            println!("{:#?}", ast);
            println!("\nReconstruction: {}", ast);
            let db: DBIndices = ast.into();
            println!("De Brujin Indices: {}", db)
        }
        Err(err) => println!("Error = {:?}", err),
    };
    Ok(())
}
