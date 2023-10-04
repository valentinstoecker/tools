use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Widget},
};

pub struct Container<W: Widget> {
    name: String,
    widget: W,
}

impl<W: Widget> Container<W> {
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
