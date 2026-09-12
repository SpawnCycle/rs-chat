use ratatui::{Frame, layout::Rect, style::Style};
use tui_logger::{TuiLoggerSmartWidget, TuiWidgetState};

pub fn draw_logs(f: &'_ mut Frame, area: Rect, state: &TuiWidgetState) {
    let w = TuiLoggerSmartWidget::default()
        .title_log("Logs")
        .title_target("Targets")
        .style_error(Style::new().red())
        .style_warn(Style::new().yellow())
        .style_info(Style::new().blue())
        .style_debug(Style::new().green())
        .style_trace(Style::new())
        .output_target(true)
        .output_file(false)
        .output_line(true)
        .state(state);

    f.render_widget(w, area);
}
