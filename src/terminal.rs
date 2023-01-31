#![allow(dead_code, unused_imports)]

use std::io::{stdout, Write};

use crossterm::terminal;
use crossterm::cursor;
use crossterm::style::{Print, PrintStyledContent};
use crossterm::{execute, queue};
use crossterm::Result;
use ropey::Rope;

// Represents the state of the terminal/cursor and implements methods for interacting with them
pub struct Terminal {
    window_length: usize,
    window_height: usize,
}

impl Terminal {
    // Create a new Terminal instance
    pub fn new() -> Self {
        // Get the terminal size
        let (window_x, window_y) = terminal::size().expect("[INTERNAL ERROR] Failed to retrieve terminal size");

        Self {
            window_length: window_x as usize,
            window_height: window_y as usize,
        }
    }

    // [Direct] Performs a frame update, clearing the screen and redrawing the buffer
    pub fn update(&self, buffer: &Rope, reset_cursor: bool) -> Result<()> {
        // Clear the screen
        self.clear(!reset_cursor, false)?;

        // Save the position of the cursor
        // This could be be either the position of the cursor at the start of the frame update,
        // Or the (0, 0) position if the cursor is being reset
        execute!(stdout(), cursor::SavePosition)?;

        // Draw the buffer, making sure to carriage return after each line
        for line in buffer.lines() {
            queue!(stdout(), Print(line), Print("\r"))?;
        }

        // Restore the cursor position to its saved state
        if reset_cursor {
            queue!(stdout(), cursor::RestorePosition)?;
        }

        stdout().flush()
    }

    // [Lazy/Direct] Clears the terminal window
    pub fn clear(&self, keep_cursor_pos: bool, direct_execute: bool) -> Result<()> {
        queue!(stdout(), terminal::Clear(terminal::ClearType::All))?;

        // The default behavior of terminal::Clear is to maintain the cursor position
        // If the user wants to reset the cursor position, it needs to be done manually
        if !keep_cursor_pos {
            queue!(stdout(), cursor::MoveTo(0, 0))?;
        }

        if direct_execute {
            stdout().flush()?;
        }

        Ok(())
    }

    // Converts a cursor position to a buffer coordinate
    fn get_buffer_coordinate(&self) -> Result<usize> {
        let (cursor_x, cursor_y) = cursor::position()?;
        Ok((cursor_y as usize) * self.window_length + cursor_x as usize)
    }
}
