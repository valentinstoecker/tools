use std::{
    env, fs,
    io::{self, Result, Stdout},
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tools::widgets::{
    container::Container,
    list::{List, ListState},
};
use tui::{
    backend::CrosstermBackend, buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget,
    Terminal,
};

#[derive(Clone)]
struct State {
    path: String,
    subdirs: List,
}

impl StatefulWidget for State {
    type State = ListState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        buf.set_string(area.x, area.y, self.path, Style::default());
        self.subdirs.render(
            Rect::new(area.x + 2, area.y + 1, area.width - 2, area.height - 1),
            buf,
            state,
        );
    }
}

struct App {
    state: State,
    list_state: ListState,
    term: Terminal<CrosstermBackend<Stdout>>,
}

impl App {
    fn init(&self) -> Result<()> {
        enable_raw_mode()?;
        Ok(())
    }

    fn reset(self) -> Result<()> {
        disable_raw_mode()?;
        let mut term = self.term;
        term.show_cursor()?;
        term.set_cursor(0, 0)?;
        term.clear()?;
        Ok(())
    }

    fn new() -> Result<Self> {
        let path = env::current_dir()?.to_str().unwrap().to_string();
        let subdirs = fs::read_dir(&path)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .map(|s| s.to_string())
            })
            .collect::<Vec<String>>();
        let subdirs = List::new(subdirs);
        let state = State { path, subdirs };
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        let list_state = ListState::default();
        let app = App {
            state,
            term,
            list_state,
        };
        app.init()?;
        Ok(app)
    }

    fn draw(&mut self) -> Result<()> {
        self.term.clear()?;
        self.term.draw(|f| {
            let widget = Container::new("vcd".to_string(), self.state.clone());
            f.render_stateful_widget(widget, f.size(), &mut self.list_state);
        })?;
        Ok(())
    }

    fn up(&mut self) {
        match self.list_state.selected {
            Some(selected) => {
                if selected > 0 {
                    self.list_state.select(selected - 1);
                }
            }
            None => {
                self.list_state.select(self.state.subdirs.len() - 1);
            }
        }
    }

    fn down(&mut self) {
        match self.list_state.selected {
            Some(selected) => {
                if selected < self.state.subdirs.len() - 1 {
                    self.list_state.select(selected + 1);
                }
            }
            None => {
                self.list_state.select(0);
            }
        }
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    loop {
        app.draw()?;
        if let Event::Key(k) = event::read()? {
            match k.code {
                KeyCode::Esc => break,
                KeyCode::Up => app.up(),
                KeyCode::Down => app.down(),
                _ => {}
            }
        }
    }
    app.reset()?;
    Ok(())
}
