mod buffer;

use buffer::vector::VectorBuffer;

use std::io::{Write, stdout};

use crossterm::{
    QueueableCommand,
    cursor::{self},
    event::{self, KeyEvent},
    execute,
    style::{self, Color, Print},
    terminal::{self, Clear, ClearType},
};

const HEADER_TEXT: &str = "Eos Text Editor";
fn main() -> std::io::Result<()> {
    let mut str = String::from("Hello, World!\nThis is a test string.\nIt has multiple lines.\n");

    let mut vector = VectorBuffer::from_string(&str)?;

    let terminate = setup_terminal(&vector)?;

    let (x, y) = cursor::position()?;

    while !terminate.load(std::sync::atomic::Ordering::Relaxed) {
        //wait for user input
        if event::poll(std::time::Duration::from_millis(100))? {
            let e = event::read()?;
            match e {
                event::Event::Key(key_event) => {
                    handle_keyboard_input(key_event, &mut vector)?;
                }
                event::Event::Mouse(event) => println!("{:?}", event),
                event::Event::FocusGained => println!("FocusGained"),
                event::Event::FocusLost => println!("FocusLost"),
                event::Event::Paste(_) => println!("Paste"),
                event::Event::Resize(_, _) => println!("Resize"),
            }
        }
    }

    //do operations on the vector

    //deserialize back to string
    str = vector.to_string()?;
    terminal::disable_raw_mode()?;
    println!("{str}");
    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(())
}

fn invert_color() -> std::io::Result<()> {
    let mut stdout = stdout();
    stdout.queue(style::SetBackgroundColor(Color::White))?;
    stdout.queue(style::SetForegroundColor(Color::Black))?;
    return Ok(());
}

fn setup_terminal(
    vector: &VectorBuffer,
) -> std::io::Result<std::sync::Arc<std::sync::atomic::AtomicBool>> {
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    terminal::enable_raw_mode()?;
    let terminate = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    signal_hook::flag::register(
        signal_hook::consts::SIGTERM,
        std::sync::Arc::clone(&terminate),
    )?;

    signal_hook::flag::register(
        signal_hook::consts::SIGINT,
        std::sync::Arc::clone(&terminate),
    )?;
    stdout.queue(cursor::MoveTo(0, 0))?;
    let (width, _) = terminal::size()?;
    invert_color()?;
    for _ in 0..width {
        stdout.queue(Print(" "))?;
    }
    stdout.queue(cursor::MoveToNextLine(1))?;
    stdout.queue(style::ResetColor)?;
    for (i, line) in vector.get_buffer().iter().enumerate() {
        stdout.queue(Print(line))?;

        if i < vector.get_buffer().len() - 1 {
            stdout.queue(cursor::MoveToNextLine(1))?;
        }
    }

    stdout.queue(cursor::SetCursorStyle::BlinkingBar)?;

    stdout.flush()?;

    Ok(terminate)
}

fn handle_keyboard_input(key_input: KeyEvent, buffer: &mut VectorBuffer) -> std::io::Result<()> {
    //figure out type of key_input
    let key_code = key_input.code;
    //2. figure out backspace/DELETE fro removal
    match key_code {
        event::KeyCode::Backspace => {
            //go for removal
            Ok(())
        }
        event::KeyCode::Up => {
            execute!(stdout(), cursor::MoveUp(1))?;
            Ok(())
        }
        event::KeyCode::Down => {
            execute!(stdout(), cursor::MoveDown(1))?;
            Ok(())
            //arrow down
        }
        event::KeyCode::Left => {
            execute!(stdout(), cursor::MoveLeft(1))?;
            Ok(())
            //arrow left
        }
        event::KeyCode::Right => {
            execute!(stdout(), cursor::MoveRight(1))?;
            Ok(())
            //arrow right
        }
        event::KeyCode::Enter => {
            //1. on enter move cursor down
            //2. figure out the string size on the move next, it can be 0, or >0
            //3. move right to next of string length
            let (x, y) = get_cursor_pos()?;
            buffer.insert_char('\n', x, y);
            //get the buffer at this point
            let buffer = buffer.get_buffer();
            let mut stdout = stdout();
            //1. print the new line to the new line below
            //2. print every new line below that to take effect
            stdout.queue(terminal::Clear(ClearType::UntilNewLine))?;
            stdout.queue(cursor::MoveToNextLine(1))?;
            stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;
            //start printing from that line to below
            for i in y..buffer.len() {
                stdout.queue(Print(&buffer[i]))?;
                stdout.queue(cursor::MoveToNextLine(1))?;
            }
            stdout.queue(cursor::MoveTo(buffer[y].len() as u16, (y + 1) as u16))?;
            stdout.flush()?;
            Ok(())
        }
        event::KeyCode::Char(c) => {
            // handle the character and its insertion
            let (x, y) = get_cursor_pos()?;
            buffer.insert_char(c, x, y);
            execute!(stdout(), Print(c))?;
            Ok(())
        }
        _ => Ok(()),
    }

    //3. otherwise simply go for insertion
}

fn get_cursor_pos() -> std::io::Result<(usize, usize)> {
    let (x, y) = cursor::position()?;
    return Ok((x as usize, y as usize));
}
