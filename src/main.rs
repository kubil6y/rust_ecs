use crate::game::Game;
use anyhow::Result;

mod game;
mod logger;

fn main() -> Result<()> {
    let mut game = Game::new("Demo")?;
    game.run()?;

    Ok(())
}
