use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::app::{App, ListItem as AppListItem};
use crate::data::{Model, Provider};

pub fn render(app: &mut App, frame: &mut Frame) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_title(frame, outer[0], app);
    render_footer(frame, outer[2], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(body[0]);

    render_search_input(frame, left[0], app);
    render_list(frame, left[1], app);
    render_detail(frame, body[1], app);
}

fn render_title(frame: &mut Frame, area: Rect, app: &App) {
    let title_text = match &app.view {
        crate::app::View::Level1 => " models.dev".to_string(),
        crate::app::View::Level2 { provider_id } => {
            if let Some(p) = app.get_provider(provider_id) {
                format!(" models.dev > {}", p.name)
            } else {
                " models.dev".to_string()
            }
        }
    };
    let title = Paragraph::new(title_text).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(title, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help = match &app.view {
        crate::app::View::Level1 => {
            " ↑↓ Navigate │ Enter: Drill into provider │ Esc: Quit │ PgUp/PgDn: Scroll details │ q: Quit"
        }
        crate::app::View::Level2 { .. } => {
            " ↑↓ Navigate │ Enter: View model │ Esc: Back │ PgUp/PgDn: Scroll details │ q: Quit"
        }
    };
    let footer = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}

fn render_search_input(frame: &mut Frame, area: Rect, app: &App) {
    let input = app.current_input();
    let width = area.width.saturating_sub(4) as usize;
    let scroll = input.visual_scroll(width);

    let search_title = match &app.view {
        crate::app::View::Level1 => " Search providers & models".to_string(),
        crate::app::View::Level2 { provider_id } => {
            if let Some(p) = app.get_provider(provider_id) {
                format!(" Search models in {}", p.name)
            } else {
                " Search models".to_string()
            }
        }
    };

    let input_widget = Paragraph::new(input.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(search_title.as_str())
                .border_style(Style::default().fg(Color::Yellow)),
        );
    frame.render_widget(input_widget, area);

    let x = input.visual_cursor().max(scroll) - scroll + 1;
    frame.set_cursor_position((area.x + x as u16, area.y + 1));
}

fn render_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ratatui::widgets::ListItem> = app
        .filtered_items
        .iter()
        .map(|item| match item {
            AppListItem::Provider { name, .. } => {
                ratatui::widgets::ListItem::new(Line::from(vec![
                    Span::styled("▸ ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        name.as_str(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" [provider]", Style::default().fg(Color::DarkGray)),
                ]))
            }
            AppListItem::Model {
                model_name,
                provider_name,
                ..
            } => {
                let mut spans = vec![Span::styled("  ", Style::default())];
                spans.push(Span::styled(
                    model_name.as_str(),
                    Style::default().fg(Color::White),
                ));
                if matches!(app.view, crate::app::View::Level1) {
                    spans.push(Span::styled(
                        format!(" ({})", provider_name),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                ratatui::widgets::ListItem::new(Line::from(spans))
            }
        })
        .collect();

    let list_title = match &app.view {
        crate::app::View::Level1 => format!(" Providers & Models ({})", items.len()),
        crate::app::View::Level2 { .. } => format!(" Models ({})", items.len()),
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title.as_str()),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_detail(frame: &mut Frame, area: Rect, app: &mut App) {
    app.detail_height = area.height.saturating_sub(2);

    let selected = app.get_selected();
    let lines = match selected {
        Some(AppListItem::Provider { id, .. }) => {
            if let Some(provider) = app.get_provider(id) {
                render_provider_detail(provider)
            } else {
                vec![Line::from("Provider not found")]
            }
        }
        Some(AppListItem::Model {
            provider_id,
            model_id,
            ..
        }) => {
            if let Some(model) = app.get_model(provider_id, model_id) {
                let provider = app.get_provider(provider_id);
                render_model_detail(model, provider, provider_id)
            } else {
                vec![Line::from("Model not found")]
            }
        }
        None => vec![Line::from("")],
    };

    // Compute wrapped line count for scroll clamping
    let content_width = area.width.saturating_sub(2) as usize; // minus borders
    let total_lines: u16 = lines
        .iter()
        .map(|line| {
            let line_width = line.width();
            if content_width == 0 {
                1u16
            } else {
                (line_width.saturating_sub(1) / content_width + 1) as u16
            }
        })
        .sum();
    app.detail_content_height = total_lines;
    app.clamp_detail_scroll();

    let detail = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(detail, area);

    // Render scrollbar on the right edge
    if app.detail_content_height > app.detail_height {
        let max_scroll = total_lines.saturating_sub(app.detail_height) as usize;
        let content_len = total_lines as usize;
        let scrollbar_pos = if max_scroll == 0 {
            0
        } else {
            app.detail_scroll as usize * content_len.saturating_sub(1) / max_scroll
        };
        let mut scrollbar_state = ScrollbarState::new(content_len)
            .viewport_content_length(app.detail_height as usize)
            .position(scrollbar_pos);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn label_value(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {}: ", label),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  ── {} ──", title),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )])
}

fn blank_line() -> Line<'static> {
    Line::from("")
}

fn bool_check(value: bool) -> &'static str {
    if value {
        "✓"
    } else {
        "✗"
    }
}

fn format_cost(cost: Option<f64>) -> String {
    match cost {
        Some(v) => format!("${:.2}", v),
        None => "—".to_string(),
    }
}

fn format_tokens(n: Option<u64>) -> String {
    match n {
        Some(v) => {
            let s = v.to_string();
            let mut result = String::new();
            for (i, c) in s.chars().rev().enumerate() {
                if i > 0 && i % 3 == 0 {
                    result.push(',');
                }
                result.push(c);
            }
            result.chars().rev().collect()
        }
        None => "—".to_string(),
    }
}

fn render_provider_detail(provider: &Provider) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            format!("  {}", provider.name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        blank_line(),
        label_value("ID", &provider.id),
        label_value("API", &provider.api),
        label_value("Documentation", &provider.doc),
        label_value("NPM Package", &provider.npm),
        label_value("Models", &provider.models.len().to_string()),
        blank_line(),
        section_header("Environment Variables"),
    ];

    for env_var in &provider.env {
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::Green)),
            Span::styled(env_var.clone(), Style::default().fg(Color::White)),
        ]));
    }

    if provider.env.is_empty() {
        lines.push(Line::from("  (none)"));
    }

    lines
}

fn render_model_detail(
    model: &Model,
    provider: Option<&Provider>,
    provider_id: &str,
) -> Vec<Line<'static>> {
    let provider_name = provider.map(|p| p.name.as_str()).unwrap_or(provider_id);

    let mut lines = vec![
        Line::from(vec![Span::styled(
            format!("  {}", model.name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        blank_line(),
        label_value("ID", &model.id),
        label_value("Provider", provider_name),
    ];

    if let Some(family) = &model.family {
        lines.push(label_value("Family", family));
    }

    lines.push(blank_line());
    lines.push(section_header("Capabilities"));

    let caps = [
        ("Reasoning", model.reasoning),
        ("Tool Call", model.tool_call),
        ("Attachment", model.attachment),
        ("Temperature", model.temperature),
    ];

    for (name, val) in caps {
        let color = if val { Color::Green } else { Color::Red };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", bool_check(val)),
                Style::default().fg(color),
            ),
            Span::styled(name, Style::default().fg(Color::White)),
        ]));
    }

    if let Some(ow) = model.open_weights {
        let color = if ow { Color::Green } else { Color::Red };
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", bool_check(ow)), Style::default().fg(color)),
            Span::styled("Open Weights", Style::default().fg(Color::White)),
        ]));
    }

    lines.push(blank_line());
    lines.push(section_header("Modalities"));
    lines.push(label_value("Input", &model.modalities.input.join(", ")));
    lines.push(label_value("Output", &model.modalities.output.join(", ")));

    if let Some(cost) = &model.cost {
        lines.push(blank_line());
        lines.push(section_header("Cost (per 1M tokens)"));
        lines.push(label_value("Input", &format_cost(cost.input)));
        lines.push(label_value("Output", &format_cost(cost.output)));
        if let Some(r) = cost.reasoning {
            lines.push(label_value("Reasoning", &format_cost(Some(r))));
        }
        if let Some(cr) = cost.cache_read {
            lines.push(label_value("Cache Read", &format_cost(Some(cr))));
        }
        if let Some(cw) = cost.cache_write {
            lines.push(label_value("Cache Write", &format_cost(Some(cw))));
        }
        if let Some(ia) = cost.input_audio {
            lines.push(label_value("Input Audio", &format_cost(Some(ia))));
        }
        if let Some(oa) = cost.output_audio {
            lines.push(label_value("Output Audio", &format_cost(Some(oa))));
        }
    }

    if let Some(limit) = &model.limit {
        lines.push(blank_line());
        lines.push(section_header("Limits"));
        if let Some(ctx) = limit.context {
            lines.push(label_value("Context", &format_tokens(Some(ctx))));
        }
        if let Some(out) = limit.output {
            lines.push(label_value("Output", &format_tokens(Some(out))));
        }
    }

    let mut meta_lines = Vec::new();
    if let Some(k) = &model.knowledge {
        meta_lines.push(label_value("Knowledge", k));
    }
    if let Some(rd) = &model.release_date {
        meta_lines.push(label_value("Release Date", rd));
    }
    if let Some(lu) = &model.last_updated {
        meta_lines.push(label_value("Last Updated", lu));
    }

    if !meta_lines.is_empty() {
        lines.push(blank_line());
        lines.push(section_header("Metadata"));
        lines.extend(meta_lines);
    }

    lines
}
