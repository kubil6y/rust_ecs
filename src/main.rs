use crate::game::{WINDOW_WIDTH, Game};
use anyhow::Result;

mod game;
mod logger;

fn main() -> Result<()> {
    println!("{}", WINDOW_WIDTH);
    let mut game = Game::new("Demo")?;
    game.run()?;

    Ok(())
}
