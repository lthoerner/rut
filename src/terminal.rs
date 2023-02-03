use std::io::{stdout, Stdout};

use crossterm::{
    cursor,
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::Paragraph,
};

use crate::Buffer;

pub struct Terminal {
    terminal: tui::Terminal<CrosstermBackend<Stdout>>,
    cursor_pos: CursorPosition,
}

impl Terminal {
    // Create a new Terminal instance
    pub fn new() -> Self {
        // Create the terminal
        let terminal = tui::Terminal::new(CrosstermBackend::new(stdout()))
            .expect("[INTERNAL ERROR] Failed to initialize terminal");

        Self {
            terminal,
            cursor_pos: CursorPosition::default(),
        }
    }

    // Open the terminal window
    pub fn open(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            DisableMouseCapture
        )
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
            f.set_cursor(self.cursor_pos.x, self.cursor_pos.y)
        })?;

        Ok(())
    }

    // Performs a cursor update
    pub fn update_cursor(&mut self) {
        execute!(
            self.terminal.backend_mut(),
            cursor::MoveTo(self.cursor_pos.x, self.cursor_pos.y)
        )
        .expect("[INTERNAL ERROR] Failed to move cursor")
    }

    // Returns a reference to the terminal's cursor
    pub fn cursor(&self) -> &CursorPosition {
        &self.cursor_pos
    }

    // Returns a mutable reference to the terminal's cursor
    pub fn cursor_mut(&mut self) -> &mut CursorPosition {
        &mut self.cursor_pos
    }
}

// Represents the position of the cursor in the buffer and in the terminal
pub struct CursorPosition {
    buffer_index: usize,
    x: u16,
    y: u16,
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self {
            buffer_index: 0,
            x: 0,
            y: 0,
        }
    }
}

impl CursorPosition {
    // Moves the cursor up
    pub fn move_up(&mut self, buffer: &Buffer) {
        let y = self.y as usize;
        let x = self.x as usize;

        // If the cursor is at the first line of the buffer, do nothing
        if y == 0 {
            return;
        }

        let prev_line_len = buffer.line_len(y - 1);

        // If the previous line is longer than the current X position,
        // move to the same X position on the previous line
        // Otherwise, move to the end of the previous line
        if prev_line_len > x {
            let extra_chars_on_previous_line = prev_line_len - x;
            self.buffer_index -= x + extra_chars_on_previous_line;
        } else {
            self.buffer_index -= x + 1;
        }

        self.update_coords(buffer);
    }

    // Moves the cursor down
    pub fn move_down(&mut self, buffer: &Buffer) {
        let y = self.y as usize;
        let x = self.x as usize;

        // If the cursor is at the last line of the buffer, do nothing
        if y == buffer.line_count() - 1 {
            return;
        }

        let next_line_len = buffer.line_len(y + 1);
        let remaining_chars_on_current_line = buffer.line_len(y) - x;

        // If the next line is longer than the current X position,
        // move to the same X position on the next line
        // Otherwise, move to the end of the next line
        if next_line_len > x {
            self.buffer_index += remaining_chars_on_current_line + x;
        } else {
            // If the next line's length is 0 but the program has not tripped a guard clause,
            // it means that the last line of the buffer is empty, which requires a special case
            self.buffer_index += remaining_chars_on_current_line + next_line_len - match next_line_len {
                0 => 0,
                _ => 1,
            };
        }

        self.update_coords(buffer);
    }

    // Moves the cursor left
    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.buffer_index > 0 {
            self.buffer_index -= 1;
        }

        self.update_coords(buffer);
    }

    // Moves the cursor right
    pub fn move_right(&mut self, buffer: &Buffer) {
        if self.buffer_index < buffer.size() {
            self.buffer_index += 1;
        }

        self.update_coords(buffer);
    }

    // Moves the cursor to teh start of the word
    pub fn move_word_left(&mut self, buffer: &Buffer) {
        if self.buffer_index > 0 {
            self.buffer_index = buffer.start_of_word(self.buffer_index);
        }

        self.update_coords(buffer);
    }

    // Moves the cursor to the end of the word
    pub fn move_word_right(&mut self, buffer: &Buffer) {
        if self.buffer_index < buffer.size() {
            self.buffer_index = buffer.end_of_word(self.buffer_index);
        }

        self.update_coords(buffer);
    }

    // Gets the cursor coordinate from its current buffer index
    fn update_coords(&mut self, buffer: &Buffer) {
        (self.x, self.y) = buffer
            .cursor_coord(self.buffer_index)
            .expect("[INTERNAL ERROR] Cursor position was out of bounds");
    }

    // Returns the cursor's buffer index
    pub fn index(&self) -> usize {
        self.buffer_index
    }
}
