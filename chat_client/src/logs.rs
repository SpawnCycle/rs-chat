use ratatui::{Frame, layout::Rect, style::Style};
use tui_logger::{TuiLoggerSmartWidget, TuiWidgetState};

pub fn draw_logs(f: &'_ mut Frame, area: Rect, state: &TuiWidgetState) {
    let w = TuiLoggerSmartWidget::default()
        .style_error(Style::new().red())
        .state(state);

    f.render_widget(w, area);
}
