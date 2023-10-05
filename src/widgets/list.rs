use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::StatefulWidget,
};

#[derive(Clone)]
pub struct ListState {
    pub selected: Option<usize>,
    offset: usize,
}

impl ListState {
    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            selected: None,
            offset: 0,
        }
    }
}

#[derive(Clone)]
pub struct List {
    items: Vec<String>,
}

impl List {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.items.get(index)
    }

    pub fn get_sel(&self, state: &ListState) -> Option<&String> {
        state.selected.and_then(|i| self.get(i))
    }
}

impl StatefulWidget for List {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // check if the selected item is visible
        if let Some(selected) = state.selected {
            if selected < state.offset {
                state.offset = selected;
            } else if selected >= state.offset + area.height as usize {
                state.offset = selected - area.height as usize + 1;
            }
        }

        // render items
        let rendered_items = self
            .items
            .iter()
            .enumerate()
            .skip(state.offset)
            .take(area.height as usize)
            .map(|(i, item)| {
                let mut style = Style::default();
                if Some(i) == state.selected {
                    style = style.fg(Color::Yellow);
                }
                (item, style)
            });
        for (i, (item, style)) in rendered_items.enumerate() {
            buf.set_string(area.x, area.y + i as u16, item, style);
        }
    }
}
