use std::{
    fs::{File, OpenOptions},
    sync::{Arc, Mutex},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    Result,
};

use crate::Buffer;
use crate::DeletionMode;
use crate::Terminal;

// Represents the state of the editor
// There should only be one instance of this struct at any given point
pub struct Editor {
    file: Arc<Mutex<File>>,
    buffer: Buffer,
    terminal: Terminal,
}

impl Editor {
    // Create a new Editor instance
    pub fn new(filename: &str) -> Self {
        // Open the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .expect("[INTERNAL ERROR] Failed to open file");

        // Read the file into the buffer
        let buffer = Buffer::new(&file);

        // Store the file in an Arc<Mutex> so it can be shared between threads
        let file = Arc::new(Mutex::new(file));

        // Create the terminal
        let terminal = Terminal::new();

        Self {
            file,
            buffer,
            terminal,
        }
    }

    // Opens the editor in the terminal and runs the event loop
    pub fn run(&mut self) -> Result<()> {
        // Open the terminal
        self.terminal.open()?;

        // Draw the initial buffer
        self.terminal.update_frame(&self.buffer)?;

        // Start the event loop
        self.start_event_loop()
    }

    // Enters the event loop for the editor
    fn start_event_loop(&mut self) -> Result<()> {
        loop {
            // Wait for the next event
            // * This is a blocking call
            let event = event::read()?;

            // Dispatch the event to the appropriate handler
            self.handle_event(event)?;
        }
    }

    // Handles a generic Event by dispatching it to the appropriate handler function
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event)?,
            _ => (),
        }

        Ok(())
    }

    // Handles a KeyEvent using its code and modifiers
    fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        match (event.code, event.modifiers) {
            // Exit the program on Ctrl+C
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.exit()?;
            }
            // Save the file on Ctrl+S
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.save()?;
            }
            // Handle arrow keypresses
            // TODO: Deduplicate and find some way to not pass in the buffer to the cursor methods
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.terminal.cursor_mut().move_up(&self.buffer);
                self.terminal.update_cursor();
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                self.terminal.cursor_mut().move_down(&self.buffer);
                self.terminal.update_cursor();
            }
            (KeyCode::Left, KeyModifiers::NONE) => {
                self.terminal.cursor_mut().move_left(&self.buffer);
                self.terminal.update_cursor();
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                self.terminal.cursor_mut().move_right(&self.buffer);
                self.terminal.update_cursor();
            }
            // Handle Ctrl+LEFT and Ctrl+RIGHT
            (KeyCode::Left, KeyModifiers::CONTROL) => {
                self.terminal.cursor_mut().move_word_left(&self.buffer);
                self.terminal.update_cursor();
            }
            (KeyCode::Right, KeyModifiers::CONTROL) => {
                self.terminal.cursor_mut().move_word_right(&self.buffer);
                self.terminal.update_cursor();
            }
            // Handle backspace
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                self.remove_char(DeletionMode::Backspace)?
            }
            // Handle Ctrl+BACKSPACE
            // ! This is bound to Ctrl+L for now because Ctrl+BACKSPACE does not seem to work
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                self.remove_word()?
            }
            // Handle delete
            (KeyCode::Delete, KeyModifiers::NONE) => self.remove_char(DeletionMode::Delete)?,
            // Handle enter
            (KeyCode::Enter, KeyModifiers::NONE) => self.insert_char('\n')?,
            // Handle normal characters
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => self.insert_char(c)?,
            _ => (),
        }

        Ok(())
    }

    // Inserts a character into the buffer at the cursor position
    fn insert_char(&mut self, character: char) -> Result<()> {
        // Get the index at which the character should be inserted
        let buffer_index = self.terminal.cursor().index();

        // Insert the character into the buffer
        self.buffer.insert(buffer_index, character);

        // Adjust the cursor position
        self.terminal.cursor_mut().move_right(&self.buffer);

        // Update the terminal
        self.terminal.update_frame(&self.buffer)?;
        self.terminal.update_cursor();

        Ok(())
    }

    // Deletes the character in the buffer immediately preceding the cursor,
    // or alternatively immediately after the cursor (delete_mode)
    fn remove_char(&mut self, deletion_mode: DeletionMode) -> Result<()> {
        use DeletionMode::*;

        // Get the index at which the character should be deleted, adjusting for the deletion mode
        let mut buffer_index = self.terminal.cursor().index();

        // Avoid backspacing characters preceding the start of the buffer
        if buffer_index == 0 && deletion_mode == Backspace {
            return Ok(());
        }

        // If a backspace is being performed, delete the character before the cursor instead of after
        if let Backspace = deletion_mode {
            buffer_index -= 1;
        }

        // Delete the character from the buffer
        self.buffer.delete(buffer_index..buffer_index + 1);

        // Adjust the cursor position depending on the deletion mode
        if let Backspace = deletion_mode {
            self.terminal.cursor_mut().move_left(&self.buffer);
        }

        // Update the terminal
        self.terminal.update_frame(&self.buffer)?;
        self.terminal.update_cursor();

        Ok(())
    }

    // Deletes the word immediately preceding the cursor
    fn remove_word(&mut self) -> Result<()> {
        // Get the index range of the word that should be deleted
        let word_end = self.terminal.cursor().index();
        let word_start = self.buffer.start_of_word(word_end);

        // Delete the word from the buffer
        self.buffer.delete(word_start..word_end);

        // Adjust the cursor position
        // TODO: Probably not the most efficient way to do this, maybe add a parameter to cursor move methods
        for _ in word_start..word_end {
            self.terminal.cursor_mut().move_left(&self.buffer);
        }

        // Update the terminal
        self.terminal.update_frame(&self.buffer)?;
        self.terminal.update_cursor();

        Ok(())
    }

    // Saves the buffer to the file
    // ! This might crash the program if the file is being saved twice at the same time
    fn save(&mut self) -> Result<()> {
        // Clone the buffer so it can be used in a separate thread
        let buffer = self.buffer.clone();

        // Get a copy of the File reference to use it in the thread
        let file = self.file.clone();

        std::thread::spawn(move || {
            // Acquire a lock on the file so it can be written to
            let mut file = file
                .lock()
                .expect("[INTERNAL ERROR] Failed to acquire lock on file");

            // Write the buffer to the file
            buffer
                .write_to_file(&mut *file)
                .expect("[INTERNAL ERROR] Failed to write to file");
        });

        Ok(())
    }

    // Closes the terminal and exits the program
    fn exit(&mut self) -> Result<()> {
        // Close the terminal
        self.terminal.exit()?;

        // Exit the program
        std::process::exit(0);
    }
}
