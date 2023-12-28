use ratatui::{prelude::*, widgets::*};

#[derive(Default)]
pub struct StatefulList {
    state: ListState,
    last_item_count: usize,
}

impl StatefulList {
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn next(&mut self) {
        if self.last_item_count == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.last_item_count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.last_item_count == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.last_item_count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render_to(&mut self, frame: &mut Frame, target: Rect, list: List) {
        self.last_item_count = list.len();
        frame.render_stateful_widget(list, target, &mut self.state);
    }
}
