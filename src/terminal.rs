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
    window_length: u16,
    window_height: u16,
}

// Represents the direction of a cursor movement
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
}

impl Terminal {
    // Create a new Terminal instance
    pub fn new() -> Self {
        // Get the terminal size
        let (window_length, window_height) = terminal::size().expect("[INTERNAL ERROR] Failed to retrieve terminal size");

        Self {
            window_length,
            window_height,
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

        // Move the cursor to the top left corner in order to draw the buffer properly
        // This may seem unnecessary, but it is actually required because if the cursor position
        // is not being reset, then the buffer would otherwise be drawn starting at the cursor position,
        // which would offset the entire frame
        queue!(stdout(), cursor::MoveTo(0, 0))?;

        // Draw the buffer, making sure to carriage return after each line
        for line in buffer.lines() {
            queue!(stdout(), Print("\r"), Print(line))?;
        }

        // Restore the cursor position to its saved state
        queue!(stdout(), cursor::RestorePosition)?;

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

    // [Direct] Deletes the character in the buffer immediately preceding the cursor,
    // or alternatively immediately after the cursor (delete_mode)
    pub fn remove_char(&self, buffer: &mut Rope, delete_mode: bool) -> Result<()> {
        let buffer_coordinate = self.get_buffer_coordinate()?;
        let buffer_len = buffer.len_chars();

        // Avoid deleting characters outside of the buffer
        match delete_mode {
            false => if buffer_coordinate == 0 || buffer_coordinate > buffer_len {
                return Ok(());
            },
            true => if buffer_coordinate >= buffer_len {
                return Ok(());
            },
        }

        // The character to delete will either be before the cursor (backspace), or after (delete)
        let remove_range = match delete_mode {
            false => buffer_coordinate - 1..buffer_coordinate,
            true => buffer_coordinate..buffer_coordinate + 1,
        };

        // Delete the character in the buffer
        buffer.remove(remove_range);

        // Perform a frame update
        self.update(buffer, false)?;

        // Move the cursor left
        self.move_cursor(CursorMovement::Left)
    }

    // [Direct] Moves the cursor in the terminal window, with wrapping
    pub fn move_cursor(&self, movement: CursorMovement) -> Result<()> {
        let (cursor_x, cursor_y) = cursor::position()?;

        use CursorMovement::*;

        match movement {
            Up => {
                // Avoid moving cursor out of bounds
                if cursor_y > 0 {
                    queue!(stdout(), cursor::MoveUp(1))?;
                } else {
                    return Ok(());
                }
            },
            Down => {
                // Avoid moving cursor out of bounds
                if cursor_y < self.window_height - 1 {
                    queue!(stdout(), cursor::MoveDown(1))?;
                } else {
                    return Ok(());
                }
            },
            Left => {
                // Avoid wrapping past the start of the screen
                // * This might need to be changed once scrolling/margins are implemented
                if cursor_x == 0 && cursor_y == 0 {
                    return Ok(());
                } else if cursor_x > 0 {
                    queue!(stdout(), cursor::MoveLeft(1))?;
                } else {
                    queue!(stdout(), cursor::MoveTo(self.window_length as u16, cursor_y - 1))?;
                }
            },
            Right => {
                let max_x = self.window_length - 1;
                let max_y = self.window_height - 1;

                // Avoid wrapping past the start of the screen
                // * This might need to be changed once scrolling/margins are implemented
                if cursor_x == max_x && cursor_y == max_y {
                    return Ok(());
                } else if cursor_x < max_x {
                    queue!(stdout(), cursor::MoveRight(1))?;
                } else {
                    queue!(stdout(), cursor::MoveTo(0, cursor_y + 1))?;
                }
            },
        }

        stdout().flush()
    }

    // Converts a cursor position to a buffer coordinate
    // * This will need to be adjusted once scrolling/margins are implemented
    fn get_buffer_coordinate(&self) -> Result<usize> {
        let (cursor_x, cursor_y) = self.get_cursor_position()?;
        Ok(cursor_y * self.window_length as usize + cursor_x)
    }

    // Essentially the same as cursor::position(), but returns usize instead of u16
    fn get_cursor_position(&self) -> Result<(usize, usize)> {
        let position = cursor::position()?;
        Ok((position.0 as usize, position.1 as usize))
    }
}
