use crossterm::terminal;
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::disable_raw_mode,
};
use std::io::{self, Write};
use std::{process, time::Duration};

enum Mode {
    Normal,
    Insert,
}

struct Editor {
    cursor_x: usize,
    cursor_y: usize,
    buffer: Vec<String>,
    mode: Mode,
}

impl Editor {
    fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            buffer: vec![String::new()],
            mode: Mode::Normal,
        }
    }

    fn draw<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        queue!(stdout, cursor::MoveTo(0, 0), cursor::Hide)?;

        let (cols, rows) = crossterm::terminal::size()?;
        let max_lines = rows as usize;

        for (i, line) in self.buffer.iter().enumerate() {
            if line.is_empty() {
                queue!(stdout, style::Print("~"))?;
            } else {
                queue!(stdout, style::Print(line))?;
            }
            if i + 1 < max_lines {
                queue!(stdout, cursor::MoveToNextLine(1))?;
            }
        }

        let cx = self.cursor_x.min((cols as usize).saturating_sub(1));
        let cy = self.cursor_y.min((rows as usize).saturating_sub(1));

        queue!(
            stdout,
            cursor::MoveTo(cx as u16, cy as u16),
            cursor::Show,
            cursor::SetCursorStyle::SteadyBlock,
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn insert_char(&mut self, c: char) {
        if let Some(line) = self.buffer.get_mut(self.cursor_y) {
            line.insert(self.cursor_x, c);
            self.cursor_x += 1;
        }
    }

    fn backspace(&mut self) {
        if let Some(line) = self.buffer.get_mut(self.cursor_y) {
            if self.cursor_x > 0 {
                line.remove(self.cursor_x - 1);
                self.cursor_x -= 1;
            } else if self.cursor_y > 0 {
                // Join with previous line
                let current_line = self.buffer.remove(self.cursor_y);
                self.cursor_y -= 1;
                if let Some(prev_line) = self.buffer.get_mut(self.cursor_y) {
                    let prev_len = prev_line.len();
                    prev_line.push_str(&current_line);
                    self.cursor_x = prev_len;
                }
            }
        }
    }

    fn handle_event(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('i') => self.mode = Mode::Insert,
                KeyCode::Char('q') => {
                    disable_raw_mode().unwrap();
                    execute!(
                        io::stdout(),
                        terminal::LeaveAlternateScreen,
                        DisableMouseCapture,
                        cursor::Show
                    )
                    .unwrap();
                    process::exit(0);
                }
                _ => {}
            },

            Mode::Insert => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Char(c) => self.insert_char(c),
                KeyCode::Backspace => self.backspace(),
                KeyCode::Enter => {
                    if let Some(line) = self.buffer.get_mut(self.cursor_y) {
                        let new_line = line.split_off(self.cursor_x);
                        self.buffer.insert(self.cursor_y + 1, new_line);
                        self.cursor_y += 1;
                        self.cursor_x = 0;
                    }
                }
                KeyCode::Left => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    } else if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                        self.cursor_x = self.buffer[self.cursor_y].len();
                    }
                }
                KeyCode::Right => {
                    if let Some(line) = self.buffer.get(self.cursor_y) {
                        if self.cursor_x < line.len() {
                            self.cursor_x += 1;
                        } else if self.cursor_y + 1 < self.buffer.len() {
                            self.cursor_y += 1;
                            self.cursor_x = 0;
                        }
                    }
                }
                KeyCode::Up => {
                    if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                        self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y].len());
                    }
                }
                KeyCode::Down => {
                    if self.cursor_y + 1 < self.buffer.len() {
                        self.cursor_y += 1;
                        self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y].len());
                    }
                }
                _ => {}
            },
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;

    let mut editor = Editor::new();

    loop {
        editor.draw(&mut stdout)?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                editor.handle_event(key);
            }
        }
    }
}
