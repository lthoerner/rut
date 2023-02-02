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

    pub fn char_after_cursor(&self, cursor_coords: (u16, u16)) -> Option<char> {
        Some(' ')
    }

    // // Returns the starting buffer index of a given line
    // // ! What happens if a line is wrapped to a new line?
    // fn line_start_index(&self, line: usize) -> usize {
    //     let mut index = 0;

    //     for (i, line_text) in self.lines().enumerate() {
    //         if i == line {
    //             return index;
    //         } else {
    //             index += line_text.len_chars();
    //         }
    //     }

    //     unreachable!(
    //         "[INTERNAL ERROR] Attempted to get the start index of a line that doesn't exist"
    //     )
    // }

    // // Converts a cursor position to a buffer coordinate
    // // * This will need to be adjusted once scrolling/margins are implemented
    // pub fn get_buffer_index(&self, (cursor_x, cursor_y): (usize, usize)) -> Option<usize> {
    //     // Check for out-of-bounds errors for the cursor Y-coordinate
    //     if cursor_y >= self.line_count() {
    //         return None;
    //     }

    //     // Get the length of the line the cursor is on
    //     // This must be done after getting the line length to avoid crashing on out-of-bounds lines
    //     // TODO: Write logic to prevent line_length() from crashing
    //     let line_length = self.line_length(cursor_y);

    //     // Check for out-of-bounds errors for the cursor X-coordinate
    //     if cursor_x > line_length {
    //         return None;
    //     }

    //     // Get the starting buffer index of the line the cursor is on
    //     let line_start = self.line_start_index(cursor_y);

    //     // Get the buffer index of the cursor
    //     Some(line_start + cursor_x)
    // }

    // // Returns the length (end X-coordinate) of a line in the buffer
    // pub fn line_length(&self, line: usize) -> usize {
    //     // TODO: Make this not convert to a String (probably semi-inefficent)
    //     let line = self.get_line(line).to_string();

    //     // If the line ends with a newline, don't count it
    //     if line.ends_with('\n') {
    //         line.len() - 1
    //     } else {
    //         line.len()
    //     }
    // }

    // // Get the number of lines in the buffer
    // pub fn line_count(&self) -> usize {
    //     self.rope.len_lines()
    // }

    // // Get the number of characters in the buffer
    // pub fn size(&self) -> usize {
    //     self.rope.len_chars()
    // }

    // // Returns an iterate over the lines in the buffer
    // pub fn lines(&self) -> ropey::iter::Lines {
    //     self.rope.lines()
    // }

    // // Returns a line from the buffer
    // // TODO: Add error handling here, as Rope.line() will panic if the line doesn't exist
    // fn get_line(&self, line: usize) -> ropey::RopeSlice {
    //     self.rope.line(line)
    // }
}
