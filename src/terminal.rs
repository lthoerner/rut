#![allow(unused_imports)]

use std::io::{stdout, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor,
    Result,
};

use tui::{backend::{CrosstermBackend, Backend}, widgets::Paragraph};

use crate::Buffer;

pub struct Terminal {
    terminal: tui::Terminal<CrosstermBackend<Stdout>>,
    window_width: u16,
    window_height: u16,
    cursor_x: u16,
    cursor_y: u16,
}

impl Terminal {
    // Create a new Terminal instance
    pub fn new() -> Self {
        // Create the terminal
        let terminal = tui::Terminal::new(CrosstermBackend::new(stdout()))
            .expect("[INTERNAL ERROR] Failed to initialize terminal");

        let window_size = terminal.size().expect("[INTERNAL ERROR] Failed to get terminal size");
        let window_width = window_size.width;
        let window_height = window_size.height;

        Self {
            terminal,
            window_width,
            window_height,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    // Open the terminal window
    pub fn open(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen, DisableMouseCapture)
    }

    // Close the terminal window
    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        self.terminal.backend_mut().show_cursor()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
    }

    // Performs a frame update
    // ? Is there a way to update the cursor without an entire frame update?
    pub fn update_frame(&mut self, buffer: &Buffer) -> Result<()> {
        let block = Paragraph::new(buffer.to_string());

        self.terminal.draw(|f| {
            // Draw the buffer
            let size = f.size();
            f.render_widget(block, size);

            // Update the cursor
            f.set_cursor(self.cursor_x, self.cursor_y)
        })?;

        Ok(())
    }

    // Performs a cursor update
    pub fn update_cursor(&mut self) {
        execute!(self.terminal.backend_mut(), cursor::MoveTo(self.cursor_x, self.cursor_y)).expect("[INTERNAL ERROR] Failed to move cursor")
    }

    // Moves the cursor up
    pub fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
        }

        self.update_cursor()
    }

    // Moves the cursor down
    pub fn move_cursor_down(&mut self) {
        if self.cursor_y < self.window_height {
            self.cursor_y += 1;
        }

        self.update_cursor()
    }

    // Moves the cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }

        self.update_cursor()
    }

    // Moves the cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_x < self.window_width {
            self.cursor_x += 1;
        }

        self.update_cursor()
    }
}
