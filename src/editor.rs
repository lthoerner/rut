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
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.terminal.cursor().move_up(&self.buffer);
                self.terminal.update_cursor();
            },
            (KeyCode::Down, KeyModifiers::NONE) => {
                self.terminal.cursor().move_down(&self.buffer);
                self.terminal.update_cursor();
            },
            (KeyCode::Left, KeyModifiers::NONE) => {
                self.terminal.cursor().move_left(&self.buffer);
                self.terminal.update_cursor();
            },
            (KeyCode::Right, KeyModifiers::NONE) => {
                self.terminal.cursor().move_right(&self.buffer);
                self.terminal.update_cursor();
            },
            // Handle backspace
            (KeyCode::Backspace, KeyModifiers::NONE) => self.remove_char(DeletionMode::Backspace)?,
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

    // [Direct] Inserts a character into the buffer at the cursor position
    fn insert_char(&mut self, character: char) -> Result<()> {
        // // Get the buffer coordinate of the cursor
        // // This should automatically avoid inserting characters outside of the buffer
        // let buffer_coordinate = match self.terminal.get_current_buffer_index() {
        //     Some(coord) => coord,
        //     None => return Ok(()),
        // };

        let buffer_index: usize = 0;

        // Insert the character into the buffer
        self.buffer.insert(buffer_index, character);

        // Move the cursor right if the character is not a newline, and move it down if it is
        match character {
            '\n' => self.terminal.cursor().move_down(&self.buffer),
            _ => self.terminal.cursor().move_right(&self.buffer),
        }

        self.terminal.update_frame(&self.buffer)?;
        self.terminal.update_cursor();

        Ok(())
    }

    // [Direct] Deletes the character in the buffer immediately preceding the cursor,
    // or alternatively immediately after the cursor (delete_mode)
    fn remove_char(&mut self, deletion_mode: DeletionMode) -> Result<()> {
        // // Get the buffer coordinate of the cursor
        // // This should automatically avoid deleting characters outside of the buffer
        // let buffer_coordinate = match self.terminal.get_current_buffer_index() {
        //     Some(coord) => coord,
        //     None => return Ok(()),
        // };
        // // Delete the character in the buffer      // // The character to delete will either be before the cursor (backspace), or after (delete)
        // self.buffer.delete(match delete_mode {
        //     true => buffer_coordinate,
        //     false => buffer_coordinate - 1,
        // });

        // // Perform a frame update
        // self.terminal.update()?;

        // Move the cursor left (backspace) or leave it in the same place (delete)

        let buffer: usize = 0;

        self.buffer.delete(buffer);
        self.terminal.update_frame(&self.buffer)?;

        match deletion_mode {
            DeletionMode::Backspace => self.terminal.cursor().move_left(&self.buffer),
            DeletionMode::Delete => (),
        }

        self.terminal.update_cursor();

        Ok(())
    }

    // [Direct] Saves the buffer to the file
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

    // [Direct] Closes the terminal and exits the program
    fn exit(&mut self) -> Result<()> {
        // Close the terminal
        self.terminal.exit()?;

        // Exit the program
        std::process::exit(0);
    }
}
