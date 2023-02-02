#![allow(unused_imports)]

use std::io::{stdout, Stdout};

use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

use tui::{backend::CrosstermBackend, widgets::Paragraph};

use crate::Buffer;

pub struct Terminal {
    terminal: tui::Terminal<CrosstermBackend<Stdout>>,
}

impl Terminal {
    // Create a new Terminal instance
    pub fn new() -> Self {
        // Create the terminal
        let terminal = tui::Terminal::new(CrosstermBackend::new(stdout()))
            .expect("[INTERNAL ERROR] Failed to initialize terminal");

        Self { terminal }
    }

    // Open the terminal window
    pub fn open(&mut self) -> Result<()> {
        enable_raw_mode()?;

        execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            // ? Is mouse capture enabled by default?
            DisableMouseCapture,
            DisableBlinking,
            Show,
        )
    }

    // Close the terminal window
    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            EnableBlinking
        )
    }

    // Performs a frame update
    pub fn update(&mut self, buffer: &Buffer) -> Result<()> {
        // ? Is there a way to do this without cloning the buffer?
        let block = Paragraph::new(buffer.to_string());

        self.terminal.draw(|f| {
            let size = f.size();
            f.render_widget(block, size);
        })?;

        Ok(())
    }
}
