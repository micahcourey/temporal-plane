use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::{
    data::{BrowseMode, MemoryEntry, retrieval_mode_label},
    state::{AppState, FocusPane},
};
use mnemix_core::memory::MemoryKind;

const MIN_HEIGHT_FOR_DETAILED_STATUS: u16 = 20;

pub(crate) fn render(frame: &mut Frame, state: &AppState) {
    let show_detailed_status = should_render_detailed_status(state, frame.area().height);
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(status_height(show_detailed_status)),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(24),
            Constraint::Length(42),
            Constraint::Min(48),
        ])
        .split(root[0]);

    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Min(8),
        ])
        .split(columns[0]);

    render_modes(frame, state, sidebar[0]);
    render_retrieval_modes(frame, state, sidebar[1]);
    render_scopes(frame, state, sidebar[2]);
    render_results(frame, state, columns[1]);
    render_detail(frame, state, columns[2]);
    render_status(frame, state, root[1], show_detailed_status);
    render_footer(frame, state, root[2]);
}

fn render_modes(frame: &mut Frame, state: &AppState, area: Rect) {
    let items = AppState::mode_options()
        .iter()
        .map(|mode| ListItem::new(mode.label()))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default().with_selected(Some(state.selected_mode_index()));
    let list = List::new(items)
        .block(
            Block::default()
                .title("Modes")
                .borders(Borders::ALL)
                .border_style(border_style(state.focus() == FocusPane::Modes)),
        )
        .highlight_style(selected_style())
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_scopes(frame: &mut Frame, state: &AppState, area: Rect) {
    let items = state
        .scopes()
        .iter()
        .map(|scope| ListItem::new(scope.label().to_owned()))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default().with_selected(Some(state.selected_scope_index()));
    let list = List::new(items)
        .block(
            Block::default()
                .title("Scopes")
                .borders(Borders::ALL)
                .border_style(border_style(state.focus() == FocusPane::Scopes)),
        )
        .highlight_style(selected_style())
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_retrieval_modes(frame: &mut Frame, state: &AppState, area: Rect) {
    let items = AppState::retrieval_mode_options()
        .iter()
        .map(|mode| {
            let label = retrieval_mode_label(*mode);
            let suffix = state
                .vector_summary()
                .unavailable_reason(*mode)
                .map_or("", |_| " (unavailable)");
            ListItem::new(format!("{label}{suffix}"))
        })
        .collect::<Vec<_>>();
    let mut list_state =
        ListState::default().with_selected(Some(state.selected_retrieval_mode_index()));
    let list = List::new(items)
        .block(
            Block::default()
                .title("Search Mode")
                .borders(Borders::ALL)
                .border_style(border_style(state.focus() == FocusPane::Retrieval)),
        )
        .highlight_style(selected_style())
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_results(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(result_title(state))
        .borders(Borders::ALL)
        .border_style(border_style(state.focus() == FocusPane::Results));

    let filtered = state.filtered_entries();
    if filtered.is_empty() {
        let message = match state.selected_mode() {
            BrowseMode::Search if state.search_query_is_empty() => {
                "Search is empty.\n\nPress `/` to enter a query."
            }
            BrowseMode::Search => "No search results match the current filters.",
            BrowseMode::Recent => "No memories match the current scope.",
            BrowseMode::Pinned => "No pinned memories match the current scope.",
        };
        frame.render_widget(
            Paragraph::new(message)
                .block(block)
                .wrap(Wrap { trim: false }),
            area,
        );
        return;
    }

    let items = filtered
        .iter()
        .map(|entry| {
            let record = entry.record();
            ListItem::new(vec![
                Line::from(format!(
                    "{} · {}{}",
                    record.title(),
                    memory_kind_label(record.kind()),
                    entry
                        .search_match()
                        .map_or_else(String::new, |search_match| {
                            search_match
                                .label()
                                .map_or_else(String::new, |label| format!(" · {label}"))
                        })
                ))
                .bold(),
                Line::from(format!(
                    "{} | {} | {}",
                    entry.updated_date(),
                    record.scope_id().as_str(),
                    record.summary()
                ))
                .dim(),
            ])
        })
        .collect::<Vec<_>>();

    let mut list_state = ListState::default().with_selected(Some(state.selected_result_index()));
    let list = List::new(items)
        .block(block)
        .highlight_style(selected_style())
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_detail(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title("Detail")
        .borders(Borders::ALL)
        .border_style(border_style(state.focus() == FocusPane::Detail));

    let Some(entry) = state.current_entry() else {
        frame.render_widget(
            Paragraph::new("No memory selected.")
                .block(block)
                .wrap(Wrap { trim: false }),
            area,
        );
        return;
    };

    frame.render_widget(
        Paragraph::new(detail_text(entry))
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((state.scroll(), 0)),
        area,
    );
}

fn render_status(frame: &mut Frame, state: &AppState, area: Rect, show_detailed_status: bool) {
    let headline = format!(
        "Mode: {} | Search mode: {} ({}) | Scope: {} | Query: {} | Updated from: {} | Updated to: {} | Results: {}",
        state.selected_mode().label(),
        state.selected_retrieval_mode_label(),
        if state.selected_retrieval_mode_supported() {
            "available"
        } else {
            "unavailable"
        },
        state.selected_scope().label(),
        display_filter_value(&state.search_filters().query),
        display_filter_value(&state.search_filters().date_from),
        display_filter_value(&state.search_filters().date_to),
        state.result_count(),
    );
    let status = if show_detailed_status {
        format!(
            "{headline}\n{}",
            state.vector_summary().detailed_snapshot_line()
        )
    } else {
        format!(
            "{headline} | {}",
            state.vector_summary().compact_snapshot_line()
        )
    };
    frame.render_widget(
        Paragraph::new(status)
            .block(Block::default().borders(Borders::TOP).title("Status"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_footer(frame: &mut Frame, state: &AppState, area: Rect) {
    let content = if let Some(field) = state.input_field() {
        format!(
            "Editing {}: {} | Enter save | Esc cancel | Backspace delete",
            field.label(),
            state.input_buffer()
        )
    } else {
        state.status_message().unwrap_or(
            "Tab focus | j/k move | / query | f from | t to | r refresh | R reload vectors | Enter or l focus detail | Esc or h back | q quit",
        )
        .to_owned()
    };
    frame.render_widget(
        Paragraph::new(content).block(Block::default().borders(Borders::TOP).title("Help")),
        area,
    );
}

fn detail_text(entry: &MemoryEntry) -> Text<'static> {
    let record = entry.record();
    let tags = join_iter(record.tags().iter().map(mnemix_core::TagName::as_str));
    let entities = join_iter(
        record
            .entities()
            .iter()
            .map(mnemix_core::EntityName::as_str),
    );
    let metadata = if record.metadata().is_empty() {
        vec![Line::from("Metadata: (none)").dim()]
    } else {
        let mut lines = vec![Line::from("Metadata").bold()];
        lines.extend(
            record
                .metadata()
                .iter()
                .map(|(key, value)| Line::from(format!("- {key}: {value}"))),
        );
        lines
    };

    let mut lines = vec![
        Line::from(record.title().to_owned()).bold(),
        Line::from(vec![
            Span::raw("Kind: "),
            Span::styled(
                memory_kind_label(record.kind()).to_owned(),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | Scope: "),
            Span::raw(record.scope_id().as_str().to_owned()),
        ]),
        Line::from(format!(
            "Updated: {} | Created: {}",
            entry.updated_date(),
            entry.created_date()
        )),
        Line::from(format!(
            "Importance: {} | Confidence: {}",
            record.importance().value(),
            record.confidence().value()
        )),
        Line::from(format!(
            "Pinned: {}",
            if record.pin_state().is_pinned() {
                "yes"
            } else {
                "no"
            }
        )),
    ];

    if let Some(reason) = record.pin_state().reason() {
        lines.push(Line::from(format!("Pin reason: {reason}")));
    }

    if let Some(search_match) = entry.search_match() {
        if let Some(label) = search_match.label() {
            lines.push(Line::from(format!("Search match: {label}")));
        }
        if let Some(score) = search_match.semantic_score() {
            lines.push(Line::from(format!("Semantic score: {score}")));
        }
    }

    lines.push(Line::from(format!("Tags: {}", display_filter_value(&tags))));
    lines.push(Line::from(format!(
        "Entities: {}",
        display_filter_value(&entities)
    )));

    if let Some(source_session_id) = record.source_session_id() {
        lines.push(Line::from(format!(
            "Source session: {}",
            source_session_id.as_str()
        )));
    }
    if let Some(source_tool) = record.source_tool() {
        lines.push(Line::from(format!("Source tool: {}", source_tool.as_str())));
    }
    if let Some(source_ref) = record.source_ref() {
        lines.push(Line::from(format!("Source ref: {}", source_ref.as_str())));
    }

    lines.push(Line::default());
    lines.push(Line::from("Summary").bold());
    lines.push(Line::from(record.summary().to_owned()));
    lines.push(Line::default());
    lines.push(Line::from("Detail").bold());
    lines.push(Line::from(record.detail().to_owned()));
    lines.push(Line::default());
    lines.extend(metadata);

    Text::from(lines)
}

fn result_title(state: &AppState) -> String {
    match state.selected_mode() {
        BrowseMode::Search if state.search_query_is_empty() => "Results".to_owned(),
        BrowseMode::Search => format!("Search Results ({})", state.selected_retrieval_mode_label()),
        BrowseMode::Recent => "Recent Memories".to_owned(),
        BrowseMode::Pinned => "Pinned Memories".to_owned(),
    }
}

fn memory_kind_label(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Observation => "observation",
        MemoryKind::Decision => "decision",
        MemoryKind::Preference => "preference",
        MemoryKind::Summary => "summary",
        MemoryKind::Fact => "fact",
        MemoryKind::Procedure => "procedure",
        MemoryKind::Warning => "warning",
    }
}

fn border_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}

fn selected_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn display_filter_value(value: &str) -> String {
    if value.is_empty() {
        "none".to_owned()
    } else {
        value.to_owned()
    }
}

fn join_iter<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let collected = values.collect::<Vec<_>>();
    if collected.is_empty() {
        String::new()
    } else {
        collected.join(", ")
    }
}

fn should_render_detailed_status(state: &AppState, area_height: u16) -> bool {
    area_height >= MIN_HEIGHT_FOR_DETAILED_STATUS && state.vector_summary().show_detailed_snapshot()
}

const fn status_height(show_detailed_status: bool) -> u16 {
    if show_detailed_status { 3 } else { 2 }
}

#[cfg(test)]
mod tests {
    use mnemix_core::{MemoryId, MemoryRecord, RecordedAt, ScopeId, memory::MemoryKind};

    use super::{should_render_detailed_status, status_height};
    use crate::tui::{
        data::{BrowserData, MemoryEntry, ScopeOption, VectorSummary},
        state::AppState,
    };

    fn memory(scope: &str) -> MemoryEntry {
        let record = MemoryRecord::builder(
            MemoryId::new(format!("memory:{scope}")).expect("id"),
            ScopeId::new(scope).expect("scope"),
            MemoryKind::Decision,
        )
        .title("Title")
        .expect("title")
        .summary("Summary")
        .expect("summary")
        .detail("Detail")
        .expect("detail")
        .updated_at(RecordedAt::new(std::time::UNIX_EPOCH))
        .build()
        .expect("record");
        MemoryEntry::from_record(record)
    }

    fn state_with_vectors(vectors_enabled: bool) -> AppState {
        let recent = vec![memory("repo:test")];
        let scopes = vec![
            ScopeOption::all(),
            ScopeOption::new(ScopeId::new("repo:test").expect("scope")),
        ];
        AppState::new(
            BrowserData::from_parts(recent.clone(), recent, scopes),
            VectorSummary::from_parts(vectors_enabled, vectors_enabled, vectors_enabled),
        )
    }

    #[test]
    fn detailed_status_collapses_for_short_terminals() {
        let state = state_with_vectors(true);

        assert!(!should_render_detailed_status(&state, 18));
        assert_eq!(status_height(false), 2);
    }

    #[test]
    fn detailed_status_expands_for_vector_enabled_tall_terminals() {
        let state = state_with_vectors(true);

        assert!(should_render_detailed_status(&state, 24));
        assert_eq!(status_height(true), 3);
    }

    #[test]
    fn detailed_status_stays_collapsed_when_vectors_are_disabled() {
        let state = state_with_vectors(false);

        assert!(!should_render_detailed_status(&state, 24));
    }
}
