#![allow(dead_code)]

use std::fs::File;
use std::io::Seek;

use crossterm::Result;
use ropey::Rope;

#[derive(Default, Clone)]
// Represents the buffer of the editor
// Basically a wrapper class for Rope to simplify/extend functionality
pub struct Buffer {
    rope: Rope,
}

pub enum DeletionMode {
    Delete,
    Backspace,
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        self.rope.to_string()
    }
}

impl Buffer {
    // Create a new Buffer instance from a File
    pub fn new(file: &File) -> Self {
        // Read the file into a Rope
        let rope = Rope::from_reader(file).expect("[INTERNAL ERROR] Failed to read file");

        Self { rope }
    }

    // Writes the buffer to the given file
    pub fn write_to_file(&self, file: &mut File) -> Result<()> {
        // Truncate the file and rewind to prepare it for writing
        file.set_len(0)?;
        file.rewind()?;

        self.rope.write_to(file)
    }

    // Inserts a character at the given index
    pub fn insert(&mut self, index: usize, character: char) {
        self.rope.insert_char(index, character);
    }

    // Deletes a character at the given index
    pub fn delete(&mut self, index: usize) {
        // Make sure the index is valid
        if index >= self.rope.len_chars() {
            return;
        }

        self.rope.remove(index..index + 1);
    }

    // Get the current cursor coordinate from a given buffer index
    pub fn cursor_coord(&self, index: usize) -> Option<(u16, u16)> {
        // Make sure the index is valid
        if index >= self.rope.len_chars() {
            return None;
        }
        
        let mut current_line: usize = 0;
        let mut current_line_start = 0;

        for (i, c) in self.rope.chars().enumerate() {
            // If the index is reached, return the current coordinate
            if i == index {
                // [EXAMPLE] if the searched index is 53, and the current_line_start
                // is 50, then the coordinate would be (3, current_line)
                return Some(((i - current_line_start) as u16, current_line as u16));
            }
            
            if c == '\n' {
                current_line += 1;
                current_line_start = i + 1;
            }
        }

        unreachable!("[INTERNAL ERROR] The given index was out of bounds but was not caught by the guard clause")
    }

    // Get the number of characters in the buffer
    pub fn size(&self) -> usize {
        self.rope.len_chars()
    }
}
