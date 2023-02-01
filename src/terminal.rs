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
    pub fn update(&self, buffer: &Rope) -> Result<()> {
        // Clear everything after the buffer
        self.partial_clear(buffer.len_chars())?;

        // Save the position of the cursor
        // This could be be either the position of the cursor at the start of the frame update,
        // Or the (0, 0) position if the cursor is being reset
        execute!(stdout(), cursor::SavePosition)?;

        // Move the cursor to the top left corner in order to draw the buffer properly
        // This may seem unnecessary, but it is actually required because if the cursor position
        // is not being reset, then the buffer would otherwise be drawn starting at the cursor position,
        // which would offset the entire frame
        self.reset_cursor()?;

        // Draw the buffer, making sure to carriage return after each line
        for line in buffer.lines() {
            queue!(stdout(), Print("\r"), Print(line))?;
        }

        // Restore the cursor position to its saved state
        queue!(stdout(), cursor::RestorePosition)?;

        stdout().flush()
    }

    // [Direct] Clears the entire terminal window
    pub fn full_clear(&self) -> Result<()> {
        self.clear(0, true)
    }

    // [Lazy] Clears the terminal window after the buffer, preserving the cursor position
    fn partial_clear(&self, buffer_size: usize) -> Result<()> {
        self.clear(buffer_size, false)
    }

    // [Lazy/Direct] Clears the terminal window, either entirely (full clear) or just the portion after the buffer
    fn clear(&self, buffer_size: usize, full_clear: bool) -> Result<()> {
        // The default behavior of terminal::Clear is to maintain the cursor position
        // If the user wants to reset the cursor position, it needs to be done manually
        if full_clear {
            self.reset_cursor()?;
        }

        // Save the cursor position, which could either be (0, 0) for a
        // full clear, or the current position for a partial clear
        execute!(stdout(), cursor::SavePosition)?;

        // If clearing the entire screen
        if full_clear {
            // Clear the entire screen
            queue!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        } else {
            // Get the coordinate of the end of the buffer
            // ? Will this need to be adjusted based on the length of the insertion?
            let (buffer_end_x, buffer_end_y) = self.get_cursor_coord_from_buffer_coord(buffer_size)?;

            // Move the cursor to the end of the buffer and clear everything after it
            queue!(stdout(), cursor::MoveTo(buffer_end_x as u16, buffer_end_y as u16))?;
            queue!(stdout(), terminal::Clear(terminal::ClearType::FromCursorDown))?;
        }

        // Restore the cursor position
        queue!(stdout(), cursor::RestorePosition)?;

        if full_clear {
            stdout().flush()?;
        }

        Ok(())
    }

    // [Direct] Deletes the character in the buffer immediately preceding the cursor,
    // or alternatively immediately after the cursor (delete_mode)
    pub fn remove_char(&self, buffer: &mut Rope, delete_mode: bool) -> Result<()> {
        let buffer_coordinate = self.get_buffer_coord()?;
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
        self.update(buffer)?;

        // Move the cursor left
        self.move_cursor(CursorMovement::Left)
    }

    // [Direct] Inserts a character into the buffer at the cursor position
    pub fn insert_char(&self, buffer: &mut Rope, character: char) -> Result<()> {
        let buffer_coordinate = self.get_buffer_coord()?;
        let buffer_len = buffer.len_chars();

        // Avoid inserting characters outside of the buffer
        if buffer_coordinate > buffer_len {
            return Ok(());
        }

        // Insert the character into the buffer
        buffer.insert_char(buffer_coordinate, character);

        // Perform a frame update
        self.update(buffer)?;

        // Move the cursor right
        self.move_cursor(CursorMovement::Right)
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
    fn get_buffer_coord(&self) -> Result<usize> {
        let (cursor_x, cursor_y) = self.get_cursor_position()?;
        Ok(cursor_y * self.window_length as usize + cursor_x)
    }

    // Converts a buffer coordinate to a cursor position
    // * This will need to be adjusted once scrolling/margins are implemented
    fn get_cursor_coord_from_buffer_coord(&self, coordinate: usize) -> Result<(usize, usize)> {
        let cursor_x = coordinate % self.window_length as usize;
        let cursor_y = coordinate / self.window_length as usize;
        Ok((cursor_x, cursor_y))
    }

    // Essentially the same as cursor::position(), but returns usize instead of u16
    fn get_cursor_position(&self) -> Result<(usize, usize)> {
        let position = cursor::position()?;
        Ok((position.0 as usize, position.1 as usize))
    }

    // [Lazy] Resets the cursor to the top left of the terminal window
    fn reset_cursor(&self) -> Result<()> {
        queue!(stdout(), cursor::MoveTo(0, 0))
    }
}
