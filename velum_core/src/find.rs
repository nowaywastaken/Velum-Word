use serde::{Deserialize, Serialize};
use regex::Regex;

/// Search options for find and replace operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Text to find
    pub query: String,
    /// Replacement text (optional)
    pub replace: String,
    /// Match case (default: false)
    #[serde(default)]
    pub case_sensitive: bool,
    /// Match whole words only (default: false)
    #[serde(default)]
    pub whole_word: bool,
    /// Treat query as regular expression (default: false)
    #[serde(default)]
    pub regex: bool,
    /// Continue from beginning when reaching end (default: true)
    #[serde(default = "default_wrap")]
    pub wrap_around: bool,
    /// Search backward (upward) (default: false)
    #[serde(default)]
    pub search_backward: bool,
}

fn default_wrap() -> bool {
    true
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            query: String::new(),
            replace: String::new(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            wrap_around: true,
            search_backward: false,
        }
    }
}

/// Result of a single search match
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    /// Match start position (byte offset)
    pub start: usize,
    /// Match end position (byte offset)
    pub end: usize,
    /// The actual matched text
    pub matched_text: String,
}

impl SearchResult {
    /// Creates a new search result
    pub fn new(start: usize, end: usize, matched_text: String) -> Self {
        SearchResult {
            start,
            end,
            matched_text,
        }
    }

    /// Returns the length of the match
    pub fn length(&self) -> usize {
        self.end - self.start
    }
}

/// Collection of all search results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SearchResultSet {
    /// All matches found
    #[serde(default)]
    pub results: Vec<SearchResult>,
    /// Total number of matches
    #[serde(default)]
    pub total_count: usize,
    /// Currently selected match index
    #[serde(default)]
    pub current_index: Option<usize>,
}

impl SearchResultSet {
    /// Creates a new empty result set
    pub fn new() -> Self {
        SearchResultSet {
            results: Vec::new(),
            total_count: 0,
            current_index: None,
        }
    }

    /// Creates a result set from a list of results
    pub fn from_results(results: Vec<SearchResult>) -> Self {
        let total_count = results.len();
        SearchResultSet {
            results,
            total_count,
            current_index: None,
        }
    }

    /// Sets the current match index
    pub fn set_current(&mut self, index: Option<usize>) {
        self.current_index = index;
    }

    /// Gets the current match if selected
    pub fn current(&self) -> Option<&SearchResult> {
        self.current_index.and_then(|i| self.results.get(i))
    }

    /// Checks if there are any results
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
}

/// Helper function to check if a position is at a word boundary
fn is_word_boundary(text: &str, byte_offset: usize) -> bool {
    if byte_offset == 0 || byte_offset >= text.len() {
        return true;
    }

    let prev_char = text[..byte_offset].chars().last();
    let next_char = text[byte_offset..].chars().next();

    let prev_is_word = prev_char.map(is_word_char).unwrap_or(false);
    let next_is_word = next_char.map(is_word_char).unwrap_or(false);

    prev_is_word != next_is_word
}

/// Check if a character is a word character (letter, digit, or underscore)
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c.is_whitespace() == false && c != '\n' && c != '\r' && c != '\t'
}

/// Helper function to escape regex special characters if not using regex mode
fn escape_regex_pattern(query: &str) -> String {
    let mut result = String::with_capacity(query.len());
    for c in query.chars() {
        if c == '[' || c == ']' || c == '(' || c == ')' || 
           c == '.' || c == '*' || c == '+' || c == '?' ||
           c == '^' || c == '$' || c == '|' || c == '\\' ||
           c == '{' || c == '}' || c == '"' || c == '\'' {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

/// Performs a simple (non-regex) search with options
fn simple_search(
    text: &str,
    query: &str,
    from: usize,
    case_sensitive: bool,
    whole_word: bool,
    wrap_around: bool,
    backward: bool,
) -> Option<SearchResult> {
    if query.is_empty() || text.is_empty() {
        return None;
    }

    let search_text = if case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    };

    let search_query = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    let query_bytes = search_query.len();

    if backward {
        // Search backward
        let search_range = if from == 0 {
            0..0
        } else {
            0..from
        };

        // If wrap_around is enabled and we don't find anything in the backward search,
        // wrap around to the end
        let mut found = None;
        let mut start_pos = search_range.start;

        while start_pos < search_range.end {
            if let Some(pos) = search_text[start_pos..search_range.end].rfind(&search_query) {
                let absolute_pos = start_pos + pos;
                
                // Check whole word boundary
                if whole_word && !is_word_boundary(text, absolute_pos) {
                    // Not a word boundary, continue searching before this match
                    if absolute_pos == 0 {
                        break;
                    }
                    start_pos = text[..absolute_pos].len().saturating_sub(1);
                    continue;
                }

                let end_pos = absolute_pos + query_bytes;
                let matched_text = text[absolute_pos..end_pos].to_string();
                found = Some(SearchResult::new(absolute_pos, end_pos, matched_text));
                break;
            }
            break;
        }

        if found.is_none() && wrap_around && from > 0 {
            // Wrap around to end and search backward
            return simple_search(text, query, text.len(), case_sensitive, whole_word, false, true);
        }

        found
    } else {
        // Search forward
        let search_start = from.min(text.len());
        let remaining = &search_text[search_start..];

        if let Some(pos) = remaining.find(&search_query) {
            let absolute_pos = search_start + pos;
            
            // Check whole word boundary
            if whole_word && !is_word_boundary(text, absolute_pos) {
                // Not a word boundary, continue searching after this match
                return simple_search(text, query, absolute_pos + 1, case_sensitive, whole_word, false, backward);
            }

            let end_pos = absolute_pos + query_bytes;
            let matched_text = text[absolute_pos..end_pos].to_string();
            Some(SearchResult::new(absolute_pos, end_pos, matched_text))
        } else if wrap_around && from > 0 {
            // Wrap around to beginning
            simple_search(text, query, 0, case_sensitive, whole_word, false, backward)
        } else {
            None
        }
    }
}

/// Performs a regex search with options
fn regex_search(
    text: &str,
    query: &str,
    from: usize,
    case_sensitive: bool,
    whole_word: bool,
    wrap_around: bool,
    backward: bool,
) -> Option<SearchResult> {
    if query.is_empty() || text.is_empty() {
        return None;
    }

    // Build regex with appropriate flags
    let re_str = if whole_word {
        format!(r"\b{}\b", query)
    } else {
        query.to_string()
    };

    let re_str = if case_sensitive {
        re_str
    } else {
        format!(r"(?i){}", re_str)
    };

    let re = match Regex::new(&re_str) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let search_start = from.min(text.len());

    if backward {
        // Find all matches and get the last one before 'from'
        let mut matches: Vec<regex::Match> = re.find_iter(text).collect();
        
        // Filter to only matches before 'from'
        matches.retain(|m| m.start() < search_start);
        
        if let Some(m) = matches.last() {
            Some(SearchResult::new(
                m.start(),
                m.end(),
                m.as_str().to_string(),
            ))
        } else if wrap_around && from > 0 {
            // Wrap around to end and search backward
            let all_matches: Vec<regex::Match> = re.find_iter(text).collect();
            all_matches.last().map(|m| SearchResult::new(
                m.start(),
                m.end(),
                m.as_str().to_string(),
            ))
        } else {
            None
        }
    } else {
        // Find the first match at or after 'from'
        if let Some(m) = re.find(text) {
            if m.start() >= search_start {
                Some(SearchResult::new(
                    m.start(),
                    m.end(),
                    m.as_str().to_string(),
                ))
            } else {
                // Need to find next match
                let remaining = &text[search_start..];
                re.find(remaining).map(|m: regex::Match| {
                    let absolute_pos = search_start + m.start();
                    SearchResult::new(
                        absolute_pos,
                        absolute_pos + m.end() - m.start(),
                        m.as_str().to_string(),
                    )
                })
            }
        } else if wrap_around && from > 0 {
            // Wrap around to beginning
            re.find(text).map(|m: regex::Match| SearchResult::new(
                m.start(),
                m.end(),
                m.as_str().to_string(),
            ))
        } else {
            None
        }
    }
}

/// Main search function that dispatches to simple or regex search
pub fn search(
    text: &str,
    options: &SearchOptions,
    from: usize,
) -> Option<SearchResult> {
    if options.query.is_empty() || text.is_empty() {
        return None;
    }

    if options.regex {
        regex_search(
            text,
            &options.query,
            from,
            options.case_sensitive,
            options.whole_word,
            options.wrap_around,
            options.search_backward,
        )
    } else {
        simple_search(
            text,
            &options.query,
            from,
            options.case_sensitive,
            options.whole_word,
            options.wrap_around,
            options.search_backward,
        )
    }
}

/// Finds all matches in text
pub fn find_all_in_text(text: &str, options: &SearchOptions) -> SearchResultSet {
    if options.query.is_empty() || text.is_empty() {
        return SearchResultSet::new();
    }

    let mut results = Vec::new();
    let mut pos = 0usize;

    while let Some(result) = search(text, options, pos) {
        results.push(result.clone());
        pos = result.end;
        
        // Avoid infinite loop for empty matches
        if result.length() == 0 {
            pos = pos.saturating_add(1);
            if pos >= text.len() {
                break;
            }
        }
    }

    SearchResultSet::from_results(results)
}

/// Applies regex replacement with capture groups
pub fn apply_regex_replacement(text: &str, pattern: &str, replacement: &str) -> String {
    if let Ok(re) = Regex::new(pattern) {
        re.replace_all(text, replacement).to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.query, "");
        assert_eq!(options.replace, "");
        assert!(!options.case_sensitive);
        assert!(!options.whole_word);
        assert!(!options.regex);
        assert!(options.wrap_around);
        assert!(!options.search_backward);
    }

    #[test]
    fn test_search_result() {
        let result = SearchResult::new(5, 10, "hello".to_string());
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
        assert_eq!(result.matched_text, "hello");
        assert_eq!(result.length(), 5);
    }

    #[test]
    fn test_search_result_set() {
        let results = vec![
            SearchResult::new(0, 5, "hello".to_string()),
            SearchResult::new(10, 15, "world".to_string()),
        ];
        let set = SearchResultSet::from_results(results.clone());
        assert_eq!(set.total_count, 2);
        assert!(set.results.iter().eq(results.iter()));
    }

    #[test]
    fn test_simple_search_forward() {
        let text = "hello world hello";
        let result = simple_search(text, "hello", 0, false, false, true, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().start, 0);
    }

    #[test]
    fn test_simple_search_case_sensitive() {
        let text = "Hello hello HELLO";
        let result = simple_search(text, "hello", 0, true, false, true, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().start, 6);
    }

    #[test]
    fn test_simple_search_case_insensitive() {
        let text = "Hello hello HELLO";
        let result = simple_search(text, "hello", 0, false, false, true, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().start, 0);
    }

    #[test]
    fn test_simple_search_whole_word() {
        let text = "hello world hello";
        let result = simple_search(text, "ell", 0, false, true, true, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_simple_search_backward() {
        let text = "hello world hello";
        // Search from position 14 (in the middle of second "hello")
        let result = simple_search(text, "hello", 14, false, false, true, true);
        assert!(result.is_some());
        // Should find the second "hello" at position 12
        assert_eq!(result.unwrap().start, 12);
    }

    #[test]
    fn test_simple_search_wrap_around() {
        let text = "hello world";
        let result = simple_search(text, "hello", 10, false, false, true, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().start, 0);
    }

    #[test]
    fn test_simple_search_not_found() {
        let text = "hello world";
        let result = simple_search(text, "xyz", 0, false, false, true, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_query() {
        let text = "hello world";
        let result = simple_search(text, "", 0, false, false, true, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_regex_search() {
        let text = "hello123 world456";
        let result = regex_search(text, r"\d+", 0, true, false, true, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().matched_text, "123");
    }

    #[test]
    fn test_find_all_in_text() {
        let text = "hello world hello";
        let options = SearchOptions {
            query: "hello".to_string(),
            ..Default::default()
        };
        let results = find_all_in_text(text, &options);
        assert_eq!(results.total_count, 2);
        assert_eq!(results.results[0].start, 0);
        assert_eq!(results.results[1].start, 12);
    }

    #[test]
    fn test_word_boundary() {
        assert!(is_word_boundary("hello", 0));
        assert!(is_word_boundary("hello", 5));
        assert!(!is_word_boundary("hello", 2));
        assert!(is_word_boundary("hello world", 5));
    }
}
