pub fn analyze(text: &str) -> Vec<String> {
    text.to_lowercase()
    .split(|c: char| !c.is_alphabetic())
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_lowercases_and_splits() {
        assert_eq!(analyze("The QUICK brown fox!"), vec!["the", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_analyze_filters_empty() {
        assert_eq!(analyze("hello   world"), vec!["hello", "world"]);
    }

    #[test]
    fn test_analyze_empty_string() {
        assert_eq!(analyze(""), Vec::<String>::new());
    }
}
