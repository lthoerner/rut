#![allow(unused_imports)]

use std::io::{stdout, Stdout};

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::Paragraph,
};

use crate::Buffer;

pub struct Terminal {
    pub terminal: tui::Terminal<CrosstermBackend<Stdout>>,
    pub window_width: u16,
    pub window_height: u16,
    pub cursor_pos: CursorPosition,
}

impl Terminal {
    // Create a new Terminal instance
    pub fn new() -> Self {
        // Create the terminal
        let terminal = tui::Terminal::new(CrosstermBackend::new(stdout()))
            .expect("[INTERNAL ERROR] Failed to initialize terminal");

        // Get the terminal size
        let window_size = terminal
            .size()
            .expect("[INTERNAL ERROR] Failed to get terminal size");
        let window_width = window_size.width;
        let window_height = window_size.height;

        Self {
            terminal,
            window_width,
            window_height,
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
    pub fn cursor(&mut self) -> &mut CursorPosition {
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
    // ! THESE ARE TEMPORARY

    // Moves the cursor up
    pub fn move_up(&mut self, buffer: &Buffer) {
        if self.y > 0 {
            self.y -= 1;
        }
    }

    // Moves the cursor down
    pub fn move_down(&mut self, buffer: &Buffer) {
        self.y += 1;
    }

    // Moves the cursor left
    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.x > 0 {
            self.x -= 1;
        }
    }

    // Moves the cursor right
    pub fn move_right(&mut self, buffer: &Buffer) {
        self.x += 1;
    }

    // ! END TEMPORARY

    // Gets the cursor coordinate from its current buffer index
    fn update_coords(&mut self, buffer: &Buffer) {
        (self.x, self.y) = buffer.cursor_coord(self.buffer_index).expect("[INTERNAL ERROR] Cursor position was out of bounds");
    }
}
