use std::rc::Rc;

use crate::{
    auth_cache::LoginDetails,
    event::{Event, EventBus},
    main_screen::MainScreen,
    ExitState, Screen,
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Prompts the user for their credentials
pub struct LoginPrompt {
    username: String,
    password: String,
    remember: bool,
    selected: SelectedInput,
    message: &'static str,
    events: Rc<EventBus>,
}

impl LoginPrompt {
    /// Create a blank form for credentials.
    /// The given [`EventBus`] will be used to initialise the [`MainScreen`] once the user submits.
    pub fn new(events: Rc<EventBus>) -> Self {
        Self {
            events,
            username: String::new(),
            password: String::new(),
            remember: false,
            selected: SelectedInput::Username,
            message: "",
        }
    }

    /// Create a blank form with the given message.
    /// This can be used to re-prompt for authentication, etc.
    pub fn new_with_msg(events: Rc<EventBus>, message: &'static str) -> Self {
        Self {
            events,
            username: String::new(),
            password: String::new(),
            remember: false,
            selected: SelectedInput::Username,
            message,
        }
    }
}

impl Screen for LoginPrompt {
    fn draw(&mut self, frame: &mut ratatui::Frame) {
        // TODO: Ugly layout code that could probably be replaced with flexbox
        let horiz_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_percentages([25, 50, 25]))
            .split(frame.size());

        let layout = Layout::default()
            .constraints(vec![
                Constraint::Min(2),    // header
                Constraint::Length(1), // padding
                Constraint::Length(1), // username
                Constraint::Length(1), // password
                Constraint::Length(1), // remember me
                Constraint::Length(1), // padding
                Constraint::Min(3),    // message
            ])
            .split(horiz_layout[1]);

        let username_para = Paragraph::new(format!("Username: {}", self.username))
            .block(Block::new().borders(self.selected.borders_for(SelectedInput::Username)));
        let password_para =
            Paragraph::new(format!("Password: {}", "*".repeat(self.password.len())))
                .block(Block::new().borders(self.selected.borders_for(SelectedInput::Password)));
        let remember_para = Paragraph::new(format!(
            "Remember? {}",
            if self.remember { "Y" } else { "N" }
        ))
        .block(Block::new().borders(self.selected.borders_for(SelectedInput::Remember)));

        let header_para = Paragraph::new("Login")
            .block(Block::new().borders(Borders::BOTTOM))
            .alignment(Alignment::Center);

        let message_para = Paragraph::new(self.message)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(header_para, layout[0]);
        frame.render_widget(username_para, layout[2]);
        frame.render_widget(password_para, layout[3]);
        frame.render_widget(remember_para, layout[4]);
        frame.render_widget(message_para, layout[6]);
    }
    fn handle_event(&mut self, event: Event) -> Result<ExitState> {
        if let Event::Key(k) = event {
            match k.code {
                // Quit shortcuts
                KeyCode::Esc => return Ok(ExitState::Quit),
                KeyCode::Char('c') | KeyCode::Char('C') if k.modifiers == KeyModifiers::CONTROL => {
                    return Ok(ExitState::Quit);
                }

                // Navigate form fields
                KeyCode::Tab | KeyCode::Down => self.selected.down(),
                KeyCode::BackTab | KeyCode::Up => self.selected.up(),
                KeyCode::Enter if self.selected != SelectedInput::Remember => self.selected.down(),

                // Typing
                KeyCode::Char(c) if !c.is_control() => match self.selected {
                    SelectedInput::Username => self.username.push(c),
                    SelectedInput::Password => self.password.push(c),
                    SelectedInput::Remember => self.remember = !self.remember,
                },
                KeyCode::Backspace => match self.selected {
                    SelectedInput::Username => {
                        self.username.pop();
                    }
                    SelectedInput::Password => {
                        self.password.pop();
                    }
                    SelectedInput::Remember => self.remember = !self.remember,
                },

                // Submit
                KeyCode::Enter => {
                    if self.username.is_empty() {
                        self.message = "Username is empty!";
                    } else if self.password.is_empty() {
                        self.message = "Password is empty!";
                    } else {
                        return Ok(ExitState::ChangeScreen(Box::new(MainScreen::new(
                            self.events.clone(),
                            LoginDetails {
                                creds: (self.username.clone(), self.password.clone().into()),
                                remember: self.remember,
                            },
                        ))));
                    }
                }

                _ => (),
            };
        };

        Ok(ExitState::Running)
    }
}

#[derive(PartialEq, Eq)]
enum SelectedInput {
    Username,
    Password,
    Remember,
}
impl SelectedInput {
    fn up(&mut self) {
        *self = match self {
            Self::Username => Self::Remember,
            Self::Password => Self::Username,
            Self::Remember => Self::Password,
        };
    }

    fn down(&mut self) {
        *self = match self {
            Self::Username => Self::Password,
            Self::Password => Self::Remember,
            Self::Remember => Self::Username,
        };
    }

    fn borders_for(&self, inp: SelectedInput) -> Borders {
        if inp == *self {
            Borders::LEFT
        } else {
            Borders::NONE
        }
    }
}
