#![allow(dead_code)]

use std::fs::File;
use std::io::Seek;

use crossterm::Result;
use ropey::{Rope, RopeSlice};

#[derive(Default, Clone)]
// Represents the buffer of the editor
// Basically a wrapper class for Rope to simplify/extend functionality
pub struct Buffer {
    rope: Rope,
}

#[derive(PartialEq)]
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
        self.rope.remove(index..index + 1);
    }

    // Gets the current cursor coordinate from a given buffer index
    pub fn cursor_coord(&self, index: usize) -> Option<(u16, u16)> {
        // Make sure the index is valid
        if index > self.size() {
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

        // If the index is at the end of the buffer, return the last coordinate
        Some(((index - current_line_start) as u16, current_line as u16))
    }

    // Gets a line from the buffer
    // ! THIS WILL CRASH IF THE LINE IS OUT OF BOUNDS
    // TODO: Make this safe to use
    fn line(&self, line: usize) -> RopeSlice {
        self.rope.line(line)
    }

    // Gets the length of a given line
    pub fn line_len(&self, line: usize) -> usize {
        self.line(line).len_chars()
    }

    // Gets the amount of lines in the buffer
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    // Gets the number of characters in the buffer
    pub fn size(&self) -> usize {
        self.rope.len_chars()
    }
}
