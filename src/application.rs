use std::path::PathBuf;
use regex::Regex;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Span}, widgets::{Block, List, ListState}, DefaultTerminal, Frame
};
use std::rc::Rc;

use crate::filesystem::{get_directory_contents, SyntaxHighlighter, SyntaxLine};

#[derive(PartialEq, Eq)]
enum State {
    Browsing,
    Searching,
    Previewing
}

pub struct Application {
    cwd: PathBuf,
    contents: Vec<PathBuf>,
    filtered: Vec<PathBuf>,
    length: usize,
    preview_length: usize,
    selected: usize,
    preview_line: usize,
    running: bool,
    input: String,
    state: State,
    browse_state: ListState,
    preview_state: ListState,
    pub clipboard: Option<String>,
    highlighter: SyntaxHighlighter
}

impl Application {
    pub fn new(entry: PathBuf) -> Self {
        let mut application = Self {
            cwd: entry,
            contents: vec![],
            filtered: vec![],
            length: 0,
            preview_length: 0,
            selected: 0,
            preview_line: 0,
            running: true,
            input: String::new(),
            state: State::Browsing,
            browse_state: ListState::default(),
            preview_state: ListState::default(),
            clipboard: None,
            highlighter: SyntaxHighlighter::new()
        };

        match get_directory_contents(&application.cwd, &mut application.contents) {
            Ok(_) => {},
            Err(_) => {}
        };

        application.filtered = application.contents.clone();
        application.length = application.contents.len();
        application
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {

        // Layout
        let layout = Layout::default().direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                    Constraint::Percentage(30),
                    Constraint::Percentage(70)
            ]).split(terminal.get_frame().area());

        self.browse_state.select(Some(self.selected));
        match terminal.draw(|frame| self.draw(frame, layout.clone())) {
            Ok(_) => {},
            Err(_) => self.running = false
        }

        while self.running {
            self.event_loop();
            if self.state == State::Browsing { self.browse_state.select(Some(self.selected)); } else { self.browse_state.select(None); }
            if self.state == State::Previewing { self.preview_state.select(Some(self.preview_line)); } else { self.preview_state.select(None); }
            match terminal.draw(|frame| self.draw(frame, layout.clone())) {
                Ok(_) => {},
                Err(_) => self.running = false
            }
        }
    }

    fn event_loop(&mut self) {
        let e = match event::read() {
            Ok(events) => events,
            Err(_) => {
                self.running = false;
                return;
            }
        };

        match e {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                if self.state == State::Searching {
                    self.input.push(c);
                } else {
                    match c {
                        'q' => self.running = false,
                        'i' => self.state = State::Searching,
                        'j' => {
                            match self.state {
                                State::Searching => {}
                                State::Browsing => if self.length > 0 { if self.selected < self.length - 1 { self.selected += 1 }},
                                State::Previewing => if self.preview_length > 0 { if self.preview_line < self.preview_length - 1 { self.preview_line += 1 }}
                            }
                        }
                        'k' => {
                            match self.state {
                                State::Searching => {}
                                State::Browsing => if self.selected > 0 { self.selected -= 1 },
                                State::Previewing => if self.preview_line > 0 { self.preview_line -= 1 }
                            }
                        }
                        'y' => {
                            self.clipboard = Some(self.filtered[self.selected].canonicalize().unwrap().to_string_lossy().to_string());
                            self.running = false;
                        },
                        'l' => { self.state = State::Previewing; self.preview_line = 0 },
                        'h' => { self.state = State::Browsing; self.selected = 0 },
                         _  => {}
                    }
                }
            }
            KeyCode::Esc => if self.state == State::Searching {
                self.state = State::Browsing;
                self.input.clear();
            },
            KeyCode::Backspace => {
                match self.state {
                    State::Searching => { self.input.pop(); },
                    _ => self.back()
                }
            },
            KeyCode::Enter => {
                match self.state {
                    State::Searching => self.state = State::Browsing,
                    _ => self.cd(),
                }
            }
            _ => {}
        }
    }

    fn cd(&mut self) {
        if self.state == State::Browsing {
            if self.filtered[self.selected].is_dir() {
                self.cwd = self.filtered[self.selected].clone();
                let _ = get_directory_contents(&self.cwd, &mut self.contents);
                self.input.clear();
                self.selected = 0;
            }
        } else {
            if self.filtered[self.selected].is_dir() {
                let _ = get_directory_contents(&self.filtered[self.selected], &mut self.contents);
                self.cwd = self.contents[self.preview_line].clone();
                let _ = get_directory_contents(&self.cwd, &mut self.contents);
                self.input.clear();
                self.state = State::Browsing;
                self.selected = 0;
                self.preview_line = 0;
            }
        }
    }

    fn back(&mut self) {
        let original = self.cwd.clone();
        self.cwd = match self.cwd.parent() {
            Some(parent) => parent.to_path_buf(),
            None => return
        };
        let _ = get_directory_contents(&self.cwd, &mut self.contents);
        
        for (idx, item) in self.contents.iter().enumerate() {
            if *item == original {
                self.selected = idx;
                break;
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame, layout: Rc<[Rect]>) {
        let title = Line::from(format!("[ {} - [{}] ]", self.cwd.to_str().unwrap(), if self.input.is_empty() { String::from(".*") } else { self.input.clone() }).bold()).green();
        let input = match self.state {
            State::Searching => Line::from(format!("[ {} ]", self.input).white()),
            _ => Line::from(format!(" {}/{} ", self.selected + 1, self.length))
        };


        let block = Block::bordered().title(title.centered()).title_bottom(input.centered()).border_set(border::THICK);

        let r = match Regex::new(&self.input) {
            Ok(r) => r,
            Err(_) => {
                self.input.clear();
                Regex::new(".*").unwrap()
            }
        };

        self.length = 0;
        self.filtered = vec![];

        let filelist = List::from(self.contents.iter().filter(|x| match self.input.is_empty() {
            true => true,
            false => r.is_match(&x.file_name().unwrap().to_string_lossy().to_string())
        }).enumerate().map(|x| Line::from(match { self.length += 1; self.filtered.push(x.1.clone()); x.1.is_dir() } {
            true => x.1.file_name().unwrap().to_string_lossy().to_string().blue(),
            false => x.1.file_name().unwrap().to_string_lossy().to_string().white()
        })).collect()).highlight_style(Style::new()).highlight_symbol(">>").repeat_highlight_symbol(true);

        if self.selected >= self.filtered.len() {
            self.selected = 0;
        }

        let mut preview: Vec<SyntaxLine> = vec![];

        if self.filtered.len() > 0 {
            if self.filtered[self.selected].is_file() {
                preview = self.highlighter.load_file(&self.filtered[self.selected]);
            } else if self.filtered[self.selected].is_dir() {
                let mut entries: Vec<PathBuf> = vec![];
                let _ = get_directory_contents(&self.filtered[self.selected], &mut entries);
                preview = entries.iter().map(|x| SyntaxLine {
                    text: vec![x.file_name().unwrap().to_string_lossy().to_string()],
                    colour: match x.is_dir() {
                        true => vec![(10, 10, 150)],
                        false => vec![(200, 200, 200)]
                    }
                }).collect();
            }
        }

        self.preview_length = preview.len();

        let preview_list = List::from(
            preview.iter().map(|x|
                Line::from(
                    (0..x.text.len()).map(|i| x.text[i].clone().fg(
                        Color::Rgb(
                            x.colour[i].0,
                            x.colour[i].1,
                            x.colour[i].2
                        )
                    )).collect::<Vec<Span<>>>()
                )
            ).collect()
        ).highlight_style(Style::new()).highlight_symbol(">>").repeat_highlight_symbol(true);

        let block2 = if self.filtered.len() > 0 {
            Block::bordered().title(Line::from(format!("[ {} ]", self.filtered[self.selected].file_name().unwrap().to_string_lossy().to_string()).blue()).centered())
                .border_set(border::THICK)
        } else {
            Block::bordered().title(Line::from(" - ").centered()).border_set(border::THICK)
        };

        frame.render_stateful_widget(filelist.block(block), layout[0], &mut self.browse_state);
        frame.render_stateful_widget(preview_list.block(block2), layout[1], &mut self.preview_state);
    }
}
