use crate::Screen;
use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;

use std::io;
use std::panic;

/// Initialize the terminal interface.
pub fn init<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

    // Define a custom panic hook to reset the terminal properties.
    // This way, you won't have your terminal messed up if an unexpected error happens.
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        reset().expect("failed to reset the terminal");
        panic_hook(panic);
    }));

    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok(())
}

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut dyn Screen) -> Result<()> {
    terminal.draw(|frame| app.draw(frame))?;

    Ok(())
}

/// Resets the terminal interface.
pub fn reset() -> Result<()> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Exits the terminal interface.
pub fn exit<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    reset()?;
    terminal.show_cursor()?;
    Ok(())
}
