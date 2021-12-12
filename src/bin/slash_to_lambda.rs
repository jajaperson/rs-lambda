use io::prelude::*;
use std::io;

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let result = buffer
        .chars()
        .map(|ch| if ch == '\\' { 'λ' } else { ch })
        .collect::<String>();
    io::stdout().write(result.as_bytes())?;
    Ok(())
}
