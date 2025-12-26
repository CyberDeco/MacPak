//! Find and Replace functionality

use floem::prelude::*;
use regex::RegexBuilder;
use std::sync::{Arc, Mutex};

use crate::state::EditorState;

/// Search state for Find & Replace
pub struct SearchState {
    pub matches: Vec<(usize, usize)>,
    pub current_index: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            matches: Vec::new(),
            current_index: 0,
        }
    }
}

lazy_static::lazy_static! {
    pub static ref SEARCH_STATE: Arc<Mutex<SearchState>> = Arc::new(Mutex::new(SearchState::default()));
}

pub fn perform_search(
    content: String,
    search_text: String,
    case_sensitive: bool,
    whole_words: bool,
    use_regex: bool,
    match_count: RwSignal<usize>,
    current_match: RwSignal<usize>,
    search_status: RwSignal<String>,
) {
    if search_text.is_empty() {
        match_count.set(0);
        current_match.set(0);
        search_status.set(String::new());
        if let Ok(mut state) = SEARCH_STATE.lock() {
            state.matches.clear();
            state.current_index = 0;
        }
        return;
    }

    // Build the regex pattern
    let pattern = if use_regex {
        search_text.clone()
    } else {
        regex::escape(&search_text)
    };

    let pattern = if whole_words {
        format!(r"\b{}\b", pattern)
    } else {
        pattern
    };

    match RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .build()
    {
        Ok(regex) => {
            let matches: Vec<(usize, usize)> = regex
                .find_iter(&content)
                .map(|m| (m.start(), m.end()))
                .collect();

            let count = matches.len();

            if let Ok(mut state) = SEARCH_STATE.lock() {
                state.matches = matches;
                state.current_index = 0;
            }

            match_count.set(count);
            current_match.set(0);
            search_status.set(String::new());
        }
        Err(e) => {
            match_count.set(0);
            current_match.set(0);
            search_status.set(format!("Invalid regex: {}", e));
            if let Ok(mut state) = SEARCH_STATE.lock() {
                state.matches.clear();
            }
        }
    }
}

pub fn find_next(state: EditorState) {
    let count = state.match_count.get();
    if count == 0 {
        return;
    }

    let current = state.current_match.get();
    let next = if current + 1 >= count { 0 } else { current + 1 };

    state.current_match.set(next);

    if let Ok(mut search_state) = SEARCH_STATE.lock() {
        search_state.current_index = next;
    }
}

pub fn find_previous(state: EditorState) {
    let count = state.match_count.get();
    if count == 0 {
        return;
    }

    let current = state.current_match.get();
    let prev = if current == 0 { count - 1 } else { current - 1 };

    state.current_match.set(prev);

    if let Ok(mut search_state) = SEARCH_STATE.lock() {
        search_state.current_index = prev;
    }
}

pub fn replace_current(state: EditorState) {
    let count = state.match_count.get();
    if count == 0 {
        state.search_status.set("No matches to replace".to_string());
        return;
    }

    let replace_with = state.replace_text.get();
    let current_idx = state.current_match.get();

    if let Ok(search_state) = SEARCH_STATE.lock() {
        if let Some(&(start, end)) = search_state.matches.get(current_idx) {
            let content = state.content.get();
            let mut new_content = String::new();
            new_content.push_str(&content[..start]);
            new_content.push_str(&replace_with);
            new_content.push_str(&content[end..]);

            state.content.set(new_content.clone());
            state.modified.set(true);

            // Re-run search to update matches
            drop(search_state);
            perform_search(
                new_content,
                state.search_text.get(),
                state.case_sensitive.get(),
                state.whole_words.get(),
                state.use_regex.get(),
                state.match_count,
                state.current_match,
                state.search_status,
            );

            state.search_status.set("Replaced 1 occurrence".to_string());
        }
    }
}

pub fn replace_all(state: EditorState) {
    let search_text = state.search_text.get();
    let replace_with = state.replace_text.get();
    let case_sensitive = state.case_sensitive.get();
    let whole_words = state.whole_words.get();
    let use_regex = state.use_regex.get();

    if search_text.is_empty() {
        state.search_status.set("Nothing to replace".to_string());
        return;
    }

    // Build the regex pattern
    let pattern = if use_regex {
        search_text.clone()
    } else {
        regex::escape(&search_text)
    };

    let pattern = if whole_words {
        format!(r"\b{}\b", pattern)
    } else {
        pattern
    };

    match RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .build()
    {
        Ok(regex) => {
            let content = state.content.get();
            let count = regex.find_iter(&content).count();

            if count == 0 {
                state.search_status.set("No matches found".to_string());
                return;
            }

            let new_content = regex
                .replace_all(&content, replace_with.as_str())
                .to_string();

            state.content.set(new_content.clone());
            state.modified.set(true);

            // Re-run search
            perform_search(
                new_content,
                state.search_text.get(),
                state.case_sensitive.get(),
                state.whole_words.get(),
                state.use_regex.get(),
                state.match_count,
                state.current_match,
                state.search_status,
            );

            state
                .search_status
                .set(format!("Replaced {} occurrences", count));
        }
        Err(e) => {
            state.search_status.set(format!("Invalid regex: {}", e));
        }
    }
}
