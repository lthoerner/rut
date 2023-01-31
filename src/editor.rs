use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};
use std::io::Seek;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use crossterm::Result;
use ropey::Rope;

use crate::terminal::Terminal;

// Represents the state of the editor
// There should only be one instance of this struct at any given point
#[allow(dead_code)]
pub struct Editor {
    file: Arc<Mutex<File>>,
    buffer: Rope,
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

        // Read the file into a Rope
        let buffer = Rope::from_reader(&file).unwrap();

        // Store the file in an Arc<Mutex> so it can be shared between threads
        let file = Arc::new(Mutex::new(file));

        let terminal = Terminal::new();

        Self {
            file,
            buffer,
            terminal,
        }
    }

    // Opens the editor in the terminal and runs the event loop
    pub fn run(&mut self) -> Result<()> {
        // Enable raw mode for the terminal
        terminal::enable_raw_mode()?;

        // Clear the screen and draw the buffer
        self.terminal.update(&self.buffer, true)?;

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
            },
            // Save the file on Ctrl+S
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.save()?;
            },
            _ => (),
        }

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
            let mut file = file.lock().expect("[INTERNAL ERROR] Failed to acquire lock on file");

            // Truncate and rewind the file
            file.set_len(0).expect("[INTERNAL ERROR] Failed to truncate file");
            file.rewind().expect("[INTERNAL ERROR] Failed to rewind file");

            // Write the buffer to the file
            buffer.write_to(&*file).expect("[INTERNAL ERROR] Failed to write to file");
        });

        Ok(())
    }

    // [Direct] Closes the terminal and exits the program
    #[allow(dead_code)]
    fn exit(&self) -> Result<()> {
        // Disable raw mode so the terminal can be used normally
        terminal::disable_raw_mode()?;

        // Clear the screen
        self.terminal.clear(false, true)?;

        // Exit the program
        std::process::exit(0);
    }
}
