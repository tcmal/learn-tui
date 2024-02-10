use ratatui::{prelude::Rect, style::Stylize, text::Line, widgets::Paragraph, Frame};

use crate::{
    event::Event,
    main_screen::{panes::Pane, Action},
    store::{DownloadState, Store},
};

#[derive(Debug, Default)]
pub struct DownloadsViewer {}

impl Pane for DownloadsViewer {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect) {
        let p = Paragraph::new(
            store
                .download_queue()
                .flat_map(|(req, state)| {
                    vec![
                        vec![
                            req.orig_filename.to_string().blue(),
                            match &state {
                                DownloadState::Queued => " - Queued".gray(),
                                DownloadState::InProgress(p) => {
                                    format!(" - {:.2}%", p * 100.0).blue()
                                }
                                DownloadState::Completed => " - Completed".green(),
                                DownloadState::Errored(e) => format!(" - {e}").red(),
                            },
                        ]
                        .into(),
                        vec![req.dest.to_string().gray()].into(),
                    ]
                })
                .collect::<Vec<Line>>(),
        );

        frame.render_widget(p, area);
    }

    fn handle_event(&mut self, _: &mut Store, _: Event) -> Action {
        Action::None
    }
}
