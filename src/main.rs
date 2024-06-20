#![deny(clippy::pedantic, clippy::nursery)]

use color_eyre::Result;
use std::{io::Write, str::FromStr};

mod expression;

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    let mut buffer = String::new();

    stdout.write_all(b"Please input a mathematical expression:\n")?;

    stdin.read_line(&mut buffer)?;

    let expression = expression::Expression::from_str(&buffer)?;
    println!("{}", expression.evaluate());

    Ok(())
}
