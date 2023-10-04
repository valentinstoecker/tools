use std::{
    env,
    io::{self, Result, Stdout},
    path::PathBuf,
};

use tui::{
    backend::CrosstermBackend, buffer::Buffer, layout::Rect, style::Style, widgets::{Widget, Block, Borders},
    Terminal,
};

struct Container<W: Widget> {
    name: String,
    widget: W,
}

impl<W: Widget> Container<W> {
    fn new(name: String, widget: W) -> Self {
        Self { name, widget }
    }
}

impl<W: Widget> Widget for Container<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title(self.name).borders(Borders::ALL);
        self.widget.render(block.inner(area), buf);
        block.render(area, buf);
    }
}

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
    Ok(())
}
