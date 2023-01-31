use std::fs::{File, OpenOptions};

use ropey::Rope;

fn main() {
    // Make sure the user has provided one argument (filename to open)
    if std::env::args().len() != 2 {
        println!("Usage: rut <filename>");
        std::process::exit(1);
    }

    // Get the filename from the command line
    let filename = std::env::args().nth(1).unwrap();

    // Create and run the editor
    let mut editor = Editor::new(&filename);
    editor.run();
}

// Represents the state of the editor
// There should only be one instance of this struct at any given point
struct Editor {
    file: File,
    buffer: Rope,
}

impl Editor {
    // Create a new editor instance
    fn new(filename: &str) -> Self {
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
    fn run(&mut self) {
        todo!()
    }
}
