mod buffer;

use buffer::vector::VectorBuffer;

use std::io::{Write, stdout};

use crossterm::{
    QueueableCommand,
    cursor::{self},
    event::{self, KeyEvent},
    execute,
    style::{self, Color, Print},
    terminal::{self, ClearType},
};

const HEADER_TEXT: &str = "Eos Text Editor";
const TAB_SPACE_SIZE: u16 = 4;
fn main() -> std::io::Result<()> {
    let mut str = String::from("Hello, World!\nThis is a test string.\nIt has multiple lines.\n");

    let mut vector = VectorBuffer::from_string(&str)?;

    let terminate = setup_terminal(&vector)?;

    while !terminate.load(std::sync::atomic::Ordering::Relaxed) {
        //wait for user input
        if event::poll(std::time::Duration::from_millis(100))? {
            let (x, y) = get_cursor_pos()?;
            let e = event::read()?;
            match e {
                event::Event::Key(key_event) => {
                    handle_keyboard_input(key_event, &mut vector, x, y)?;
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
    for i in 0..vector.size() {
        stdout.queue(Print(vector.get_line(i)))?;

        if i < vector.size() - 1 {
            stdout.queue(cursor::MoveToNextLine(1))?;
        }
    }

    stdout.queue(cursor::SetCursorStyle::BlinkingBar)?;

    stdout.flush()?;

    Ok(terminate)
}

fn handle_keyboard_input(
    key_input: KeyEvent,
    buffer: &mut VectorBuffer,
    x: usize,
    y: usize,
) -> std::io::Result<()> {
    //figure out type of key_input
    let key_code = key_input.code;
    let mut stdout = stdout();
    //2. figure out backspace/DELETE fro removal
    match key_code {
        event::KeyCode::Backspace => {
            buffer.remove_char(x, y - 1);
            if x > 0 {
                let line = buffer.get_line(y - 1);

                stdout.queue(cursor::MoveLeft(1))?;
                stdout.queue(terminal::Clear(ClearType::UntilNewLine))?;

                let (i, j) = get_cursor_pos()?;

                stdout.queue(Print(&line[(x - 1)..]))?;
                stdout.queue(cursor::MoveTo(i as u16, j as u16))?;
            } else if y > 1 {
                let previous_line = buffer.get_line(y - 2);
                let previous_len = previous_line.len();

                stdout.queue(cursor::MoveTo(0, (y - 1) as u16))?;
                stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;

                for i in y - 2..buffer.size() {
                    stdout.queue(Print(buffer.get_line(i)))?;
                    stdout.queue(cursor::MoveToNextLine(1))?;
                }

                stdout.queue(cursor::MoveTo(previous_len as u16, (y - 1) as u16))?;
            }
        }
        event::KeyCode::Up => {
            //1. figure out current pos
            if y <= 1 {
                return Ok(());
            }
            let string = buffer.get_line(y - 2);
            if x > string.len() {
                stdout.queue(cursor::MoveTo(string.len() as u16, (y - 1) as u16))?;
            } else {
                stdout.queue(cursor::MoveUp(1))?;
            }
        }
        event::KeyCode::Down => {
            if y == buffer.size() {
                return Ok(());
            }
            let line = buffer.get_line(y);
            if x > line.len() {
                stdout.queue(cursor::MoveTo(line.len() as u16, (y + 1) as u16))?;
            } else {
                stdout.queue(cursor::MoveDown(1))?;
            }
            //arrow down
        }
        event::KeyCode::Left => {
            if x == 0 {
                if y > 1 {
                    stdout.queue(cursor::MoveTo(
                        buffer.get_line(y - 2).len() as u16,
                        (y - 1) as u16,
                    ))?;
                }
            } else {
                stdout.queue(cursor::MoveLeft(1))?;
            }
            //arrow left
        }
        event::KeyCode::Right => {
            //for each line venture till end then go below until that is also not possible...
            let line = buffer.get_line(y - 1);
            if x == line.len() {
                //go below
                if y < buffer.size() {
                    stdout.queue(cursor::MoveTo(0, (y + 1) as u16))?;
                }
            } else {
                stdout.queue(cursor::MoveRight(1))?;
            }
            //arrow right
        }
        event::KeyCode::Enter => {
            //1. on enter move cursor down
            //2. figure out the string size on the move next, it can be 0, or >0
            //3. move right to next of string length
            buffer.insert_char('\n', x, y);
            //get the buffer at this point
            //1. print the new line to the new line below
            //2. print every new line below that to take effect
            stdout.queue(terminal::Clear(ClearType::UntilNewLine))?;
            stdout.queue(cursor::MoveToNextLine(1))?;
            stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;
            //start printing from that line to below
            for i in y..buffer.size() {
                stdout.queue(Print(buffer.get_line(i)))?;
                stdout.queue(cursor::MoveToNextLine(1))?;
            }
            stdout.queue(cursor::MoveTo(0, (y + 1) as u16))?;
        }
        event::KeyCode::Char(c) => {
            // handle the character and its insertion
            buffer.insert_char(c, x, y);
            // execute!(stdout(), Print(c))?;
            stdout.queue(Print(c))?;
        }
        event::KeyCode::Tab => {
            for _ in 0..TAB_SPACE_SIZE {
                buffer.insert_char(' ', x, y);
            }
            stdout.queue(cursor::MoveRight(TAB_SPACE_SIZE))?;
        }
        _ => {}
    }

    stdout.flush()?;
    Ok(())
}

fn get_cursor_pos() -> std::io::Result<(usize, usize)> {
    let (x, y) = cursor::position()?;
    return Ok((x as usize, y as usize));
}
