use std::fs::{File, OpenOptions};

use ropey::Rope;
use crossterm::terminal;

// Represents the state of the editor
// There should only be one instance of this struct at any given point
#[allow(dead_code)]
pub struct Editor {
    file: File,
    buffer: Rope,
    window_length: usize,
    window_height: usize,
}

impl Editor {
    // Create a new editor instance
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

        // Get the terminal size
        let window_size = terminal::size().expect("[INTERNAL ERROR] Failed to get terminal size");

        Self {
            file,
            buffer,
            window_length: window_size.0 as usize,
            window_height: window_size.1 as usize,
        }
    }

    // Opens the editor in the terminal and runs the event loop
    pub fn run(&mut self) {
        todo!()
    }
}
