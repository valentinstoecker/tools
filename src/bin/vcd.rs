use std::{
    env,
    io::{self, Result, Stdout},
    path::PathBuf,
};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tools::widgets::container::Container;
use tui::{
    backend::CrosstermBackend, buffer::Buffer, layout::Rect, style::Style, widgets::Widget,
    Terminal,
};

#[derive(Clone)]
struct State {
    path: PathBuf,
}

impl Widget for State {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            self.path.to_str().unwrap().to_string(),
            Style::default(),
        )
    }
}

struct App {
    state: State,
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
        Ok(())
    }

    fn new() -> Result<Self> {
        let path = env::current_dir()?;
        let state = State { path };
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        let app = App { state, term };
        app.init()?;
        Ok(app)
    }

    fn draw(&mut self) -> Result<()> {
        self.term.clear()?;
        self.term.draw(|f| {
            let widget = Container::new("vcd".to_string(), self.state.clone());
            f.render_widget(widget, f.size());
        })?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.draw()?;
    app.reset()?;
    Ok(())
}
