use std::{
    env,
    io::{self, Result, Stdout},
    path::PathBuf,
};

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
    fn new() -> Result<Self> {
        let path = env::current_dir()?;
        let state = State { path };
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        Ok(App { state, term })
    }

    fn draw(&mut self) -> Result<()> {
        self.term.draw(|f| {
            f.render_widget(self.state.clone(), f.size());
        })?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.draw()?;
    Ok(())
}
