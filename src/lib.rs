#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde_json;

use std::str;
use regex::Regex;
use std::borrow::Borrow;
use serde_json::Value;

// include the json in the bin
const AFFIN: &[u8; 32811] = include_bytes!("./afinn.json");

lazy_static! {
    static ref AFFIN_VALUE: Value = {
        let json = str::from_utf8(AFFIN).unwrap();
        serde_json::from_str(json).unwrap()
    };
}

/// Struct for return the outcome of individual sentiments
pub struct Sentiment {
    /// The sentiment score
    pub score: f32,
    /// The score compared with total tokens analysed
    pub comparative: f32,
    /// The matching set of words
    pub words: Vec<String>,
}

/// Struct for return the outcome of analysis
pub struct Analysis {
    /// The sentiment score
    pub score: f32,
    /// The score compared with total tokens analysed
    pub comparative: f32,
    /// Positivity score
    pub positive: Sentiment,
    /// Negativity score
    pub negative: Sentiment,
}

fn tokenize_with_no_punctuation(phrase: &str) -> Vec<String> {
    let re = Regex::new(r"[^a-zA-Z0 -]+").unwrap();
    let re2 = Regex::new(r" {2,}").unwrap();

    let no_punctuation = re.replace_all(phrase, " ");
    let no_punctuation = re2.replace_all(no_punctuation.borrow(), " ");

    no_punctuation
        .to_lowercase()
        .split(' ')
        .map(|s| s.to_string())
        .collect()
}

/// Calculates the negativity of a sentence
pub fn negativity(phrase: String) -> Sentiment {
    let tokens = tokenize_with_no_punctuation(phrase.as_str());
    let tokens_len = tokens.len() as f32;
    let mut score = 0f32;
    let mut words = Vec::new();

    for t in tokens {
        let word = t.clone();
        if let Value::Number(ref val) = AFFIN_VALUE[t] {
            let diff = val.as_f64().unwrap() as f32;
            if diff < 0f32 {
                score -= diff;
                words.push(word);
            }
        }
    }

    Sentiment {
        score,
        comparative: score / tokens_len,
        words,
    }
}

/// Calculates the positivity of a sentence
pub fn positivity(phrase: String) -> Sentiment {
    let tokens = tokenize_with_no_punctuation(phrase.as_str());
    let tokens_len = tokens.len() as f32;
    let mut score = 0f32;
    let mut words = Vec::new();

    for t in tokens {
        let word = t.clone();
        if let Value::Number(ref val) = AFFIN_VALUE[t] {
            let diff = val.as_f64().unwrap() as f32;
            if diff > 0f32 {
                score += diff;
                words.push(word);
            }
        }
    }

    Sentiment {
        score,
        comparative: score / tokens_len,
        words,
    }
}

/// Calculates the overall sentiment
pub fn analyze(phrase: String) -> Analysis {
    let neg = negativity(phrase.clone());
    let pos = positivity(phrase.clone());

    Analysis {
        score: pos.score - neg.score,
        comparative: pos.comparative - neg.comparative,
        positive: pos,
        negative: neg,
    }
}

#[test]
fn decode_affin() {
    let json = str::from_utf8(AFFIN).unwrap();
    assert!(json.len() != 0usize);
}

#[test]
fn tokenize() {
    let tokens = tokenize_with_no_punctuation("staRt,./     {middle//////end");
    assert_eq!(
        tokens,
        vec!["start".to_string(), "middle".to_string(), "end".to_string()]
    );
}

#[test]
fn test_negativity() {
    let sentiment = negativity("I do not like jam tarts".to_string());
    assert_eq!(sentiment.score, 0f32);
    assert_eq!(sentiment.words, Vec::<String>::new());
}

#[test]
fn test_positivity() {
    let sentiment = positivity("I do not like jam tarts".to_string());
    assert_eq!(sentiment.score, 2f32);
    assert_eq!(sentiment.words, vec!["like"]);
}

#[test]
fn test_analyze() {
    let analysis = analyze("I do not like jam tarts".to_string());
    assert_eq!(analysis.score, 2f32);
}
