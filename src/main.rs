use std::io::{Write, stdout};

use crossterm::{
    QueueableCommand,
    cursor::{self, SetCursorStyle::DefaultUserShape},
    event, execute,
    style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};

const HEADER_TEXT: &str = "Eos Text Editor";

fn main() -> std::io::Result<()> {
    // setting the background color for the whole terminal
    setup_terminal()?;

    let mut str = String::from("Hello, World!\nThis is a test string.\nIt has multiple lines.\n");

    //serialize to data structure
    let mut vector = serialize_to_vector(&str)?;

    //do operations on the vector

    //deserialize back to string
    str = deserialize_vector_to_string(&vector)?;

    std::thread::sleep(std::time::Duration::from_secs(100));
    Ok(())
}

fn serialize_to_vector(str: &String) -> std::io::Result<Vec<String>> {
    let mut vector = Vec::new();
    for line in str.lines() {
        vector.push(line.to_string());
    }
    Ok(vector)
}

fn deserialize_vector_to_string(vector: &Vec<String>) -> std::io::Result<String> {
    let mut str = String::new();
    for line in vector {
        str.push_str(line);
        str.push('\n');
    }
    Ok(str)
}

fn invert_color() -> std::io::Result<()> {
    let mut stdout = stdout();
    stdout.queue(style::SetBackgroundColor(Color::White))?;
    stdout.queue(style::SetForegroundColor(Color::Black))?;
    return Ok(());
}

fn setup_terminal() -> std::io::Result<()> {
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    //add header
    //for the header we print a line in middle
    let (width, _) = terminal::size()?;
    invert_color()?;
    // stdout.queue(cursor::SetCursorStyle(DefaultUserShape))?;
    for _ in 0..width {
        stdout.queue(Print(" "))?;
    }
    stdout.queue(cursor::MoveToNextLine(1))?;
    stdout.queue(style::ResetColor)?;
    stdout.queue(cursor::EnableBlinking)?;
    stdout.flush()?;

    //add footer

    //create our spacer

    Ok(())
}
