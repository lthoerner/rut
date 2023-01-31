use std::fs::{File, OpenOptions};

use ropey::Rope;

// Represents the state of the editor
// There should only be one instance of this struct at any given point
pub struct Editor {
    file: File,
    buffer: Rope,
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

        // Return the editor
        Self { file, buffer }
    }

    // Opens the editor in the terminal and runs the event loop
    pub fn run(&mut self) {
        todo!()
    }
}