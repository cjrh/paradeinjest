const POSITIVE_WORDS: &[&str] = &[
    "good",
    "great",
    "excellent",
    "amazing",
    "wonderful",
    "fantastic",
    "happy",
    "love",
    "loved",
    "loving",
    "best",
    "better",
    "positive",
    "success",
    "successful",
    "win",
    "winning",
    "awesome",
    "perfect",
    "pleased",
    "glad",
    "thankful",
    "grateful",
    "brilliant",
    "outstanding",
    "superb",
    "helpful",
    "impressive",
    "exciting",
    "excited",
];

const NEGATIVE_WORDS: &[&str] = &[
    "bad",
    "terrible",
    "awful",
    "horrible",
    "hate",
    "hated",
    "hating",
    "worst",
    "worse",
    "negative",
    "fail",
    "failed",
    "failing",
    "failure",
    "angry",
    "sad",
    "unhappy",
    "disappointed",
    "disappointing",
    "frustrating",
    "frustrated",
    "annoying",
    "annoyed",
    "poor",
    "broken",
    "wrong",
    "problem",
    "issue",
    "bug",
    "error",
];

pub struct SentimentAnalyzer;

#[derive(Debug, Clone, PartialEq)]
pub struct SentimentResult {
    pub label: String,
    pub score: f32,
}

impl SentimentAnalyzer {
    pub fn analyze(text: &str) -> SentimentResult {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        let total_words = words.len();

        if total_words == 0 {
            return SentimentResult {
                label: "neutral".to_string(),
                score: 0.0,
            };
        }

        let mut positive_count = 0;
        let mut negative_count = 0;

        for word in &words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
            if POSITIVE_WORDS.contains(&clean_word) {
                positive_count += 1;
            }
            if NEGATIVE_WORDS.contains(&clean_word) {
                negative_count += 1;
            }
        }

        let score =
            (positive_count as f32 - negative_count as f32) / (total_words as f32).max(1.0);

        let label = if score > 0.05 {
            "positive"
        } else if score < -0.05 {
            "negative"
        } else {
            "neutral"
        };

        SentimentResult {
            label: label.to_string(),
            score: score.clamp(-1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_sentiment() {
        let result = SentimentAnalyzer::analyze("This is great and amazing work!");
        assert_eq!(result.label, "positive");
        assert!(result.score > 0.0);
    }

    #[test]
    fn test_negative_sentiment() {
        let result = SentimentAnalyzer::analyze("This is terrible and awful");
        assert_eq!(result.label, "negative");
        assert!(result.score < 0.0);
    }

    #[test]
    fn test_neutral_sentiment() {
        let result = SentimentAnalyzer::analyze("The meeting is at 3pm tomorrow");
        assert_eq!(result.label, "neutral");
    }

    #[test]
    fn test_empty_text() {
        let result = SentimentAnalyzer::analyze("");
        assert_eq!(result.label, "neutral");
        assert_eq!(result.score, 0.0);
    }
}
