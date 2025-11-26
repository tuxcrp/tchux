use std::{
    io::Write,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use crate::utils::{emoji_preprocessor, encrypt_message};
use ansi_to_tui::IntoText;
use ratatui::{
    crossterm::event::{self, Event as CEvent, KeyCode, MouseEventKind},
    init,
    layout::{Constraint, Direction, Layout},
    restore,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

struct App {
    input: String,
    messages: Vec<String>,
    scroll: i16, // Negative means scroll up ie. 0 would be to show bottom
}

pub enum AppEvent {
    Submit,
    NetMessage(String),
    Quit,
}

pub fn tui(
    tx: Sender<AppEvent>,
    rx: Receiver<AppEvent>,
    key: &[u8],
    mut stream: TcpStream,
) -> anyhow::Result<()> {
    let mut terminal = init();

    let mut app = App {
        input: String::new(),
        messages: vec![],
        scroll: 0,
    };

    let result = loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(f.area());

            let visible_height = chunks[0].height.saturating_sub(2) as i16; // subtract borders

            // Maximum scroll is when the top line is at index 0
            let max_scroll = (app.messages.len() as i16 - visible_height).max(0);

            // Clamp scroll to [0, max_scroll], store as negative for your logic
            app.scroll = app.scroll.clamp(-max_scroll, 0);

            let bottom_offset = -app.scroll; // now positive
            let start =
                (app.messages.len() as i16 - visible_height - bottom_offset).max(0) as usize;
            let end = (app.messages.len() as i16 - bottom_offset).max(0) as usize;

            let visible = &app.messages[start..end];

            let msg_box = Paragraph::new(visible.join("\n").into_text().unwrap())
                .block(Block::default().borders(Borders::ALL).title("Chat"));
            f.render_widget(msg_box, chunks[0]);

            let input_box = Paragraph::new(app.input.as_str())
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input_box, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                CEvent::Key(ev) => match ev.code {
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        tx.send(AppEvent::Submit).ok();
                    }

                    KeyCode::Up => app.scroll -= 1,
                    KeyCode::Down => app.scroll += 1,
                    KeyCode::PageUp => app.scroll -= 5,
                    KeyCode::PageDown => app.scroll += 5,
                    KeyCode::Home => app.scroll = i16::MIN,
                    KeyCode::End => app.scroll = 0,

                    KeyCode::Esc => break Ok(()),
                    _ => {}
                },
                CEvent::Mouse(me) => match me.kind {
                    MouseEventKind::ScrollUp => app.scroll -= 1,
                    MouseEventKind::ScrollDown => app.scroll += 1,
                    _ => {}
                },
                _ => {}
            }
            let max_offset = (app.messages.len() as i16).saturating_sub(1);
            app.scroll = app.scroll.clamp(-max_offset, 0);
        }

        while let Ok(ev) = rx.try_recv() {
            match ev {
                AppEvent::Submit => {
                    if !app.input.is_empty() {
                        let encrypted = encrypt_message(key, &app.input);
                        stream.write_all(encrypted.as_bytes()).ok();
                        app.input.clear();
                        app.scroll = 0;
                    }
                }
                AppEvent::NetMessage(msg) => {
                    app.messages.push(emoji_preprocessor(msg));
                    app.scroll = 0;
                }
                AppEvent::Quit => break,
            }
        }
    };

    restore();
    result
}
