mod data;
mod render;
mod state;

use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mnemix_core::QueryLimit;
use mnemix_lancedb::LanceDbBackend;
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::errors::CliError;

use self::{
    data::{BrowseMode, BrowserData, search_entries, validate_date_filter},
    render::render,
    state::{AppState, FocusPane, InputField},
};

pub(crate) fn run(backend: &LanceDbBackend, limit: QueryLimit) -> Result<(), CliError> {
    let data = BrowserData::load(backend, limit)?;
    let mut terminal = init_terminal()?;
    let result = run_app(&mut terminal, backend, limit, data);
    restore_terminal(&mut terminal)?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    backend: &LanceDbBackend,
    limit: QueryLimit,
    data: BrowserData,
) -> Result<(), CliError> {
    let mut state = AppState::new(data);

    loop {
        terminal.draw(|frame| render(frame, &state))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if should_quit(key, &state) {
            break;
        }

        if state.input_field().is_some() {
            handle_input_key(&mut state, backend, limit, key)?;
            continue;
        }

        if handle_global_key(&mut state, backend, limit, key)? {
            continue;
        }

        match state.focus() {
            FocusPane::Modes => {
                if handle_mode_key(&mut state, key) {
                    refresh_search_if_needed(&mut state, backend, limit)?;
                }
            }
            FocusPane::Scopes => {
                if handle_scope_key(&mut state, key) {
                    refresh_search_if_needed(&mut state, backend, limit)?;
                }
            }
            FocusPane::Results => handle_results_key(&mut state, key),
            FocusPane::Detail => handle_detail_key(&mut state, key),
        }
    }

    Ok(())
}

fn handle_global_key(
    state: &mut AppState,
    backend: &LanceDbBackend,
    limit: QueryLimit,
    key: KeyEvent,
) -> Result<bool, CliError> {
    match key.code {
        KeyCode::Tab => {
            state.set_focus_next();
            Ok(true)
        }
        KeyCode::BackTab => {
            state.set_focus_previous();
            Ok(true)
        }
        KeyCode::Char('/') => {
            state.begin_input(InputField::Query);
            Ok(true)
        }
        KeyCode::Char('f') => {
            state.begin_input(InputField::DateFrom);
            Ok(true)
        }
        KeyCode::Char('t') => {
            state.begin_input(InputField::DateTo);
            Ok(true)
        }
        KeyCode::Char('r') => {
            if state.selected_mode() == BrowseMode::Search {
                refresh_search_if_needed(state, backend, limit)?;
                state.set_status_message("Refreshed search results");
                Ok(true)
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}

fn handle_mode_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => state.next_mode(),
        KeyCode::Up | KeyCode::Char('k') => state.previous_mode(),
        KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
            state.set_focus_next();
            false
        }
        _ => false,
    }
}

fn handle_scope_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => state.next_scope(),
        KeyCode::Up | KeyCode::Char('k') => state.previous_scope(),
        KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
            state.set_focus_next();
            false
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.set_focus_previous();
            false
        }
        _ => false,
    }
}

fn handle_results_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => state.next_result(),
        KeyCode::Up | KeyCode::Char('k') => state.previous_result(),
        KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => state.set_focus_next(),
        KeyCode::Left | KeyCode::Char('h') => state.set_focus_previous(),
        _ => {}
    }
}

fn handle_detail_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => state.scroll_down(),
        KeyCode::Up | KeyCode::Char('k') => state.scroll_up(),
        KeyCode::Left | KeyCode::Esc | KeyCode::Char('h') => state.set_focus_previous(),
        _ => {}
    }
}

fn handle_input_key(
    state: &mut AppState,
    backend: &LanceDbBackend,
    limit: QueryLimit,
    key: KeyEvent,
) -> Result<(), CliError> {
    match key.code {
        KeyCode::Esc => state.cancel_input(),
        KeyCode::Enter => {
            let Some(field) = state.input_field() else {
                return Ok(());
            };
            if matches!(field, InputField::DateFrom | InputField::DateTo) {
                let value = state.input_buffer().trim();
                if let Err(message) = validate_date_filter(value) {
                    state.set_status_message(message);
                    return Ok(());
                }
            }
            let committed = state.commit_input();
            match committed {
                Some(InputField::Query) => {
                    refresh_search_if_needed(state, backend, limit)?;
                    if state.search_query_is_empty() {
                        state.set_status_message("Cleared search query");
                    } else {
                        state.set_status_message("Updated search query");
                    }
                }
                Some(InputField::DateFrom | InputField::DateTo) => {
                    state.reset_result_selection();
                    state.set_status_message("Updated date filters");
                }
                None => {}
            }
        }
        KeyCode::Backspace => state.pop_input_char(),
        KeyCode::Char(value) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.push_input_char(value);
        }
        _ => {}
    }
    Ok(())
}

fn refresh_search_if_needed(
    state: &mut AppState,
    backend: &LanceDbBackend,
    limit: QueryLimit,
) -> Result<(), CliError> {
    if state.selected_mode() != BrowseMode::Search {
        return Ok(());
    }

    if state.search_query_is_empty() {
        state.set_search_results(Vec::new());
        return Ok(());
    }

    let scope = state.selected_scope().value().cloned();
    let results = search_entries(backend, &state.search_filters().query, scope, limit)?;
    state.set_search_results(results);
    Ok(())
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, CliError> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), CliError> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn should_quit(key: KeyEvent, state: &AppState) -> bool {
    if state.input_field().is_some() {
        return false;
    }

    matches!(key.code, KeyCode::Char('q')) && !key.modifiers.contains(KeyModifiers::CONTROL)
}
