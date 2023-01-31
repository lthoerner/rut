mod editor;

use editor::Editor;

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
