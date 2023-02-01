// #![allow(dead_code, unused_imports)]

use std::io::{stdout, Write};

use crossterm::cursor;
use crossterm::style::Print;
use crossterm::terminal;
use crossterm::Result;
use crossterm::{execute, queue};
use ropey::Rope;

// Encapsulates TerminalState and implements methods for interacting with the terminal
// This contains a reference to the buffer because many indexing operations need to be buffer-aware,
// but it would not make sense for TerminalState to contain the buffer as it is the Editor
// that is working on the buffer, not the terminal
pub struct Terminal<'a> {
    pub state: &'a mut TerminalState,
    pub buffer: &'a Rope,
}

// Represents the state of the terminal/cursor
pub struct TerminalState {
    window_length: u16,
    window_height: u16,
}

// Represents the direction of a cursor movement
// ? Does this really need to exist?
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
}

impl TerminalState {
    // Create a new Terminal instance
    pub fn new() -> Self {
        // Get the terminal size
        let (window_length, window_height) =
            terminal::size().expect("[INTERNAL ERROR] Failed to retrieve terminal size");

        Self {
            window_length,
            window_height,
        }
    }
}

impl Terminal<'_> {
    // [Direct] Initializes the Terminal
    pub fn init(&self) -> Result<()> {
        // Enable raw mode
        terminal::enable_raw_mode()?;

        // Disable blinking cursor
        execute!(stdout(), cursor::DisableBlinking)?;

        // Clear the screen
        self.full_clear()?;

        // Draw the buffer
        self.update()
    }

    // [Direct] Performs a frame update, clearing the screen and redrawing the buffer
    pub fn update(&self) -> Result<()> {
        // Clear everything after the buffer
        self.partial_clear()?;

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
        for line in self.buffer.lines() {
            queue!(stdout(), Print("\r"), Print(line))?;
        }

        // Restore the cursor position to its saved state
        queue!(stdout(), cursor::RestorePosition)?;

        stdout().flush()
    }

    // [Direct] Clears the entire terminal window
    fn full_clear(&self) -> Result<()> {
        self.clear(true)
    }

    // [Lazy] Clears the terminal window after the buffer, preserving the cursor position
    fn partial_clear(&self) -> Result<()> {
        self.clear(false)
    }

    // [Lazy/Direct] Clears the terminal window, either entirely (full clear) or just the portion after the buffer
    fn clear(&self, full_clear: bool) -> Result<()> {
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
            let (buffer_end_x, buffer_end_y) =
                self.get_cursor_coord_from_buffer_index(self.buffer.len_chars())?;

            // Move the cursor to the end of the buffer and clear everything after it
            queue!(
                stdout(),
                cursor::MoveTo(buffer_end_x as u16, buffer_end_y as u16)
            )?;
            queue!(
                stdout(),
                terminal::Clear(terminal::ClearType::FromCursorDown)
            )?;
        }

        // Restore the cursor position
        queue!(stdout(), cursor::RestorePosition)?;

        if full_clear {
            stdout().flush()?;
        }

        Ok(())
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
            }
            Down => {
                // Avoid moving cursor out of bounds
                if cursor_y < self.state.window_height - 1 {
                    queue!(stdout(), cursor::MoveDown(1))?;
                } else {
                    return Ok(());
                }
            }
            Left => {
                // Avoid wrapping past the start of the screen
                // * This might need to be changed once scrolling/margins are implemented
                if cursor_x == 0 && cursor_y == 0 {
                    return Ok(());
                } else if cursor_x > 0 {
                    queue!(stdout(), cursor::MoveLeft(1))?;
                } else {
                    let previous_line = cursor_y - 1;
                    queue!(
                        stdout(),
                        // ? Is there a better way to do this without two type casts?
                        cursor::MoveTo(self.line_length(previous_line as usize) as u16, previous_line)
                    )?;
                }
            }
            Right => {
                let max_x = self.state.window_length - 1;
                let max_y = self.state.window_height - 1;

                // Avoid wrapping past the start of the screen
                // * This might need to be changed once scrolling/margins are implemented
                if cursor_x == max_x && cursor_y == max_y {
                    return Ok(());
                } else if cursor_x < max_x {
                    queue!(stdout(), cursor::MoveRight(1))?;
                } else {
                    queue!(stdout(), cursor::MoveTo(0, cursor_y + 1))?;
                }
            }
        }

        stdout().flush()
    }

    // Converts a cursor position to a buffer coordinate
    // * This will need to be adjusted once scrolling/margins are implemented
    // ? Should this just return Result<usize>?
    pub fn get_current_buffer_index(&self) -> Result<Option<usize>> {
        let (cursor_x, cursor_y) = self.get_cursor_position()?;

        // Check for out-of-bounds errors for the cursor Y-coordinate
        if cursor_y >= self.line_count(self.buffer) {
            return Ok(None);
        }

        // Get the length of the line the cursor is on
        // This must be done after getting the line length to avoid crashing on out-of-bounds lines
        // TODO: Write logic to prevent line_length() from crashing
        let line_length = self.line_length(cursor_y);

        // Check for out-of-bounds errors for the cursor X-coordinate
        if cursor_x > line_length {
            return Ok(None);
        }

        // Get the starting buffer index of the line the cursor is on
        let line_start = self.line_start_index(cursor_y);

        // Get the buffer index of the cursor
        Ok(Some(line_start + cursor_x))
    }

    // Returns the number of lines in the buffer
    // ? Should this be moved somewhere else?
    fn line_count(&self, buffer: &Rope) -> usize {
        buffer.len_lines()
    }

    // Returns the length (end X-coordinate) of a line in the buffer
    // ? Should this be moved somewhere else?
    fn line_length(&self, line: usize) -> usize {
        // TODO: Make this not convert to a String (probably semi-inefficent)
        let line = self.buffer.line(line).to_string();

        // If the line ends with a newline, don't count it
        if line.ends_with('\n') {
            line.len() - 1
        } else {
            line.len()
        }
    }

    // Returns the starting buffer index of a given line
    // ! What happens if a line is wrapped to a new line?
    // ? Should this be moved somewhere else?
    fn line_start_index(&self, line: usize) -> usize {
        let mut index = 0;

        for (i, line_text) in self.buffer.lines().enumerate() {
            if i == line {
                return index;
            } else {
                index += line_text.len_chars();
            }
        }

        unreachable!(
            "[INTERNAL ERROR] Attempted to get the start index of a line that doesn't exist"
        )
    }

    // Converts a buffer coordinate to a cursor position
    // * This will need to be adjusted once scrolling/margins are implemented
    // TODO: Update this for line-aware indexing
    fn get_cursor_coord_from_buffer_index(&self, coordinate: usize) -> Result<(usize, usize)> {
        let cursor_x = coordinate % self.state.window_length as usize;
        let cursor_y = coordinate / self.state.window_length as usize;
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

    // [Direct] Exits the terminal window and sets it back to its normal behavior
    pub fn exit(&self) -> Result<()> {
        // Disable raw mode so the terminal can be used normally
        terminal::disable_raw_mode()?;

        // Re-enable cursor blinking
        queue!(stdout(), cursor::EnableBlinking)?;

        // Clear the screen
        self.full_clear()
    }
}
