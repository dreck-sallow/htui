use std::time::Duration;

use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    Frame,
};

use crate::programs::tui_v2::pane::state_v2::responses::Response;

const SPACING: &'static str = "   ";

pub fn draw_line(res: &Response, area: Rect, frame: &mut Frame) {
    let version = Span::from(&res.version).gray();

    let (status, status_text) = {
        let style = match res.status {
            100..=199 => Style::default().blue(),
            200..=299 => Style::default().green(),
            300..=399 => Style::default().yellow(),
            400..=499 => Style::default().magenta(),
            500..=599 => Style::default().red(),
            _ => Style::default(),
        };

        (
            Span::from(res.status.to_string()).style(style),
            Span::from(&res.status_text).style(style),
        )
    };

    let duration = {
        let millis = res.duration.as_millis();
        let style = if millis < 200 {
            Style::default().green()
        } else if millis < 500 {
            Style::default().yellow()
        } else {
            Style::default().red()
        };

        Span::from(duration_as_str(res.duration)).style(style)
    };

    frame.render_widget(
        Line::from_iter([
            version,
            Span::raw(SPACING),
            status,
            Span::raw(" "),
            status_text,
            Span::raw(SPACING),
            duration,
            Span::raw(SPACING),
            Span::raw(&res.content_type),
        ]),
        area,
    );
}

fn duration_as_str(d: Duration) -> String {
    let millis = d.as_millis();
    let secs = d.as_secs_f64();

    if secs < 1_f64 {
        return format!("{:.3}ms", millis);
    }

    let minutes = secs / 60_f64;

    if minutes < 1_f64 {
        return format!("{:.3}s", secs);
    }

    return format!("{:.4}m", minutes);
}
