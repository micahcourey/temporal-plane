use mnemix_core::RetrievalMode;

use super::data::{
    BrowseMode, BrowserData, MemoryEntry, RETRIEVAL_MODES, ScopeOption, VectorSummary,
    matches_date_range, retrieval_mode_label,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FocusPane {
    Modes,
    Retrieval,
    Scopes,
    Results,
    Detail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputField {
    Query,
    DateFrom,
    DateTo,
}

impl InputField {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Query => "search query",
            Self::DateFrom => "updated from",
            Self::DateTo => "updated to",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SearchFilters {
    pub(crate) query: String,
    pub(crate) date_from: String,
    pub(crate) date_to: String,
}

pub(crate) struct AppState {
    data: BrowserData,
    vector_summary: VectorSummary,
    search_results: Vec<MemoryEntry>,
    selected_mode: usize,
    selected_retrieval_mode: usize,
    selected_scope: usize,
    selected_result: usize,
    scroll: u16,
    focus: FocusPane,
    input_field: Option<InputField>,
    input_buffer: String,
    status_message: Option<String>,
    search_filters: SearchFilters,
}

impl AppState {
    pub(crate) fn new(data: BrowserData, vector_summary: VectorSummary) -> Self {
        Self {
            data,
            vector_summary,
            search_results: Vec::new(),
            selected_mode: 0,
            selected_retrieval_mode: 0,
            selected_scope: 0,
            selected_result: 0,
            scroll: 0,
            focus: FocusPane::Results,
            input_field: None,
            input_buffer: String::new(),
            status_message: None,
            search_filters: SearchFilters::default(),
        }
    }

    pub(crate) fn focus(&self) -> FocusPane {
        self.focus
    }

    pub(crate) fn selected_mode(&self) -> BrowseMode {
        BrowseMode::ALL[self.selected_mode]
    }

    pub(crate) fn mode_options() -> &'static [BrowseMode] {
        &BrowseMode::ALL
    }

    pub(crate) fn retrieval_mode_options() -> &'static [RetrievalMode] {
        &RETRIEVAL_MODES
    }

    pub(crate) fn scopes(&self) -> &[ScopeOption] {
        self.data.scopes()
    }

    pub(crate) fn vector_summary(&self) -> &VectorSummary {
        &self.vector_summary
    }

    pub(crate) fn set_vector_summary(&mut self, vector_summary: VectorSummary) {
        self.vector_summary = vector_summary;
    }

    pub(crate) fn selected_retrieval_mode(&self) -> RetrievalMode {
        RETRIEVAL_MODES[self.selected_retrieval_mode]
    }

    pub(crate) fn selected_retrieval_mode_label(&self) -> &'static str {
        retrieval_mode_label(self.selected_retrieval_mode())
    }

    pub(crate) fn selected_retrieval_mode_supported(&self) -> bool {
        self.vector_summary
            .supports_mode(self.selected_retrieval_mode())
    }

    pub(crate) fn selected_retrieval_mode_unavailable_reason(&self) -> Option<String> {
        self.vector_summary
            .unavailable_reason(self.selected_retrieval_mode())
    }

    pub(crate) fn search_filters(&self) -> &SearchFilters {
        &self.search_filters
    }

    pub(crate) fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub(crate) fn input_field(&self) -> Option<InputField> {
        self.input_field
    }

    pub(crate) fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub(crate) fn selected_scope(&self) -> &ScopeOption {
        &self.data.scopes()[self.selected_scope]
    }

    pub(crate) fn set_search_results(&mut self, results: Vec<MemoryEntry>) {
        self.search_results = results;
        self.reset_result_selection();
    }

    pub(crate) fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub(crate) fn begin_input(&mut self, field: InputField) {
        self.input_field = Some(field);
        self.input_buffer = match field {
            InputField::Query => self.search_filters.query.clone(),
            InputField::DateFrom => self.search_filters.date_from.clone(),
            InputField::DateTo => self.search_filters.date_to.clone(),
        };
    }

    pub(crate) fn cancel_input(&mut self) {
        self.input_field = None;
        self.input_buffer.clear();
    }

    pub(crate) fn commit_input(&mut self) -> Option<InputField> {
        let field = self.input_field?;
        let value = self.input_buffer.trim().to_owned();
        match field {
            InputField::Query => self.search_filters.query = value,
            InputField::DateFrom => self.search_filters.date_from = value,
            InputField::DateTo => self.search_filters.date_to = value,
        }
        self.cancel_input();
        Some(field)
    }

    pub(crate) fn push_input_char(&mut self, value: char) {
        self.input_buffer.push(value);
    }

    pub(crate) fn pop_input_char(&mut self) {
        self.input_buffer.pop();
    }

    pub(crate) fn set_focus_next(&mut self) {
        self.focus = match self.focus {
            FocusPane::Modes => FocusPane::Retrieval,
            FocusPane::Retrieval => FocusPane::Scopes,
            FocusPane::Scopes => FocusPane::Results,
            FocusPane::Results => FocusPane::Detail,
            FocusPane::Detail => FocusPane::Modes,
        };
    }

    pub(crate) fn set_focus_previous(&mut self) {
        self.focus = match self.focus {
            FocusPane::Modes => FocusPane::Detail,
            FocusPane::Retrieval => FocusPane::Modes,
            FocusPane::Scopes => FocusPane::Retrieval,
            FocusPane::Results => FocusPane::Scopes,
            FocusPane::Detail => FocusPane::Results,
        };
    }

    pub(crate) fn next_mode(&mut self) -> bool {
        let previous = self.selected_mode();
        self.selected_mode = (self.selected_mode + 1) % BrowseMode::ALL.len();
        self.reset_result_selection();
        previous != self.selected_mode()
    }

    pub(crate) fn previous_mode(&mut self) -> bool {
        let previous = self.selected_mode();
        self.selected_mode =
            (self.selected_mode + BrowseMode::ALL.len() - 1) % BrowseMode::ALL.len();
        self.reset_result_selection();
        previous != self.selected_mode()
    }

    pub(crate) fn next_retrieval_mode(&mut self) -> bool {
        let previous = self.selected_retrieval_mode();
        self.selected_retrieval_mode = (self.selected_retrieval_mode + 1) % RETRIEVAL_MODES.len();
        self.reset_result_selection();
        previous != self.selected_retrieval_mode()
    }

    pub(crate) fn previous_retrieval_mode(&mut self) -> bool {
        let previous = self.selected_retrieval_mode();
        self.selected_retrieval_mode =
            (self.selected_retrieval_mode + RETRIEVAL_MODES.len() - 1) % RETRIEVAL_MODES.len();
        self.reset_result_selection();
        previous != self.selected_retrieval_mode()
    }

    pub(crate) fn next_scope(&mut self) -> bool {
        if self.data.scopes().is_empty() {
            return false;
        }
        let previous = self.selected_scope;
        self.selected_scope = (self.selected_scope + 1) % self.data.scopes().len();
        self.reset_result_selection();
        previous != self.selected_scope
    }

    pub(crate) fn previous_scope(&mut self) -> bool {
        if self.data.scopes().is_empty() {
            return false;
        }
        let previous = self.selected_scope;
        self.selected_scope =
            (self.selected_scope + self.data.scopes().len() - 1) % self.data.scopes().len();
        self.reset_result_selection();
        previous != self.selected_scope
    }

    pub(crate) fn next_result(&mut self) {
        let len = self.filtered_entries().len();
        if len == 0 {
            self.selected_result = 0;
            return;
        }
        self.selected_result = (self.selected_result + 1) % len;
        self.scroll = 0;
    }

    pub(crate) fn previous_result(&mut self) {
        let len = self.filtered_entries().len();
        if len == 0 {
            self.selected_result = 0;
            return;
        }
        self.selected_result = (self.selected_result + len - 1) % len;
        self.scroll = 0;
    }

    pub(crate) fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub(crate) fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub(crate) fn scroll(&self) -> u16 {
        self.scroll
    }

    pub(crate) fn filtered_entries(&self) -> Vec<&MemoryEntry> {
        let selected_scope = self.selected_scope().value();
        let scope_matches = |entry: &&MemoryEntry| {
            selected_scope.is_none_or(|scope| entry.record().scope_id() == scope)
        };

        match self.selected_mode() {
            BrowseMode::Recent => self.data.recent().iter().filter(scope_matches).collect(),
            BrowseMode::Pinned => self.data.pinned().iter().filter(scope_matches).collect(),
            BrowseMode::Search => self
                .search_results
                .iter()
                .filter(|entry| {
                    matches_date_range(
                        entry,
                        (!self.search_filters.date_from.is_empty())
                            .then_some(self.search_filters.date_from.as_str()),
                        (!self.search_filters.date_to.is_empty())
                            .then_some(self.search_filters.date_to.as_str()),
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn current_entry(&self) -> Option<&MemoryEntry> {
        let filtered = self.filtered_entries();
        filtered.get(self.selected_result).copied()
    }

    pub(crate) fn result_count(&self) -> usize {
        self.filtered_entries().len()
    }

    pub(crate) fn selected_result_index(&self) -> usize {
        self.selected_result
    }

    pub(crate) fn selected_scope_index(&self) -> usize {
        self.selected_scope
    }

    pub(crate) fn selected_mode_index(&self) -> usize {
        self.selected_mode
    }

    pub(crate) fn selected_retrieval_mode_index(&self) -> usize {
        self.selected_retrieval_mode
    }

    pub(crate) fn search_query_is_empty(&self) -> bool {
        self.search_filters.query.is_empty()
    }

    pub(crate) fn reset_result_selection(&mut self) {
        self.selected_result = 0;
        self.scroll = 0;
    }
}

#[cfg(test)]
mod tests {
    use mnemix_core::{MemoryId, MemoryRecord, RecordedAt, ScopeId, memory::MemoryKind};

    use super::{AppState, BrowserData, FocusPane, InputField};
    use crate::tui::data::{MemoryEntry, ScopeOption, VectorSummary};

    fn memory(scope: &str, updated_at: std::time::SystemTime) -> MemoryEntry {
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
        .updated_at(RecordedAt::new(updated_at))
        .build()
        .expect("record");
        MemoryEntry::from_record(record)
    }

    fn data() -> BrowserData {
        let recent = vec![
            memory(
                "repo:one",
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_774_867_200),
            ),
            memory(
                "repo:two",
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_775_040_000),
            ),
        ];
        let pinned = recent.clone();
        let scopes = vec![
            ScopeOption::all(),
            ScopeOption::new(ScopeId::new("repo:one").expect("scope")),
            ScopeOption::new(ScopeId::new("repo:two").expect("scope")),
        ];
        BrowserData::from_parts(recent, pinned, scopes)
    }

    #[test]
    fn focus_cycles_across_five_panes() {
        let mut state = AppState::new(data(), VectorSummary::from_parts(false, false, false));
        assert_eq!(state.focus(), FocusPane::Results);
        state.set_focus_next();
        assert_eq!(state.focus(), FocusPane::Detail);
        state.set_focus_next();
        assert_eq!(state.focus(), FocusPane::Modes);
        state.set_focus_next();
        assert_eq!(state.focus(), FocusPane::Retrieval);
        state.set_focus_previous();
        assert_eq!(state.focus(), FocusPane::Modes);
    }

    #[test]
    fn search_date_filters_reduce_visible_results() {
        let mut state = AppState::new(data(), VectorSummary::from_parts(false, false, false));
        state.next_mode();
        state.next_mode();
        state.set_search_results(vec![
            memory(
                "repo:one",
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_774_867_200),
            ),
            memory(
                "repo:two",
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_775_040_000),
            ),
        ]);
        state.begin_input(InputField::DateFrom);
        for character in "2026-04-01".chars() {
            state.push_input_char(character);
        }
        let _ = state.commit_input();

        assert_eq!(state.result_count(), 1);
    }

    #[test]
    fn retrieval_mode_support_uses_vector_summary() {
        let mut state = AppState::new(data(), VectorSummary::from_parts(true, false, false));
        assert_eq!(state.selected_retrieval_mode_label(), "Lexical");
        assert!(state.selected_retrieval_mode_supported());

        state.next_retrieval_mode();
        assert_eq!(state.selected_retrieval_mode_label(), "Semantic");
        assert!(!state.selected_retrieval_mode_supported());
        assert!(
            state
                .selected_retrieval_mode_unavailable_reason()
                .expect("reason")
                .contains("embedding provider")
        );
    }

    #[test]
    fn vector_summary_can_be_reloaded_in_state() {
        let mut state = AppState::new(data(), VectorSummary::from_parts(true, false, false));
        state.next_retrieval_mode();
        assert!(!state.selected_retrieval_mode_supported());

        state.set_vector_summary(VectorSummary::from_parts(true, true, true));

        assert!(state.selected_retrieval_mode_supported());
    }
}
