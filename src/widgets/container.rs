use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, StatefulWidget, Widget},
};

pub struct Container<W> {
    name: String,
    widget: W,
}

impl<W> Container<W> {
    pub fn new(name: String, widget: W) -> Self {
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

impl<S: StatefulWidget> StatefulWidget for Container<S> {
    type State = S::State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default().title(self.name).borders(Borders::ALL);
        self.widget.render(block.inner(area), buf, state);
        block.render(area, buf);
    }
}
