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

fn read_dir(path: &str) -> Result<Vec<String>> {
    let mut subdirs = fs::read_dir(path)?
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
    subdirs.sort();
    Ok(subdirs)
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
        let subdirs = List::new(Vec::new());
        let state = State {
            path: "".to_string(),
            subdirs,
        };
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        let list_state = ListState::default();
        let mut app = App {
            state,
            term,
            list_state,
        };
        app.init()?;
        app.set_dir(path)?;
        Ok(app)
    }

    fn set_dir(&mut self, path: String) -> Result<()> {
        let subdirs = read_dir(&path)?;
        self.state.path = path;
        self.state.subdirs = List::new(subdirs);
        self.list_state.select(0);
        Ok(())
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

    fn left(&mut self) {
        let path = self.state.path.clone();
        let path = path.split("/").collect::<Vec<&str>>();
        let path = path[..path.len() - 1].join("/");
        self.set_dir(path).unwrap();
    }

    fn right(&mut self) {
        let path = self.state.path.clone();
        if let Some(selected) = self.state.subdirs.get_sel(&self.list_state) {
            let path = format!("{}/{}", path, selected);
            let _ = self.set_dir(path);
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
                KeyCode::Left => app.left(),
                KeyCode::Right => app.right(),
                _ => {}
            }
        }
    }
    app.reset()?;
    Ok(())
}
