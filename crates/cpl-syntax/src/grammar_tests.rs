//! The generated editor grammars classify every word of std/ and the
//! examples as the lexer does.

use crate::{grammar, lex, TokenKind, LITERAL_WORDS};
use std::collections::BTreeMap;
use std::path::Path;

/// The words of each `syntax keyword GROUP ...` line of Vim's grammar.
fn vim_words() -> BTreeMap<String, &'static str> {
    let mut out = BTreeMap::new();
    for line in grammar::vim().lines() {
        let mut words = line.split_whitespace();
        if let (Some("syntax"), Some("keyword"), Some(group)) =
            (words.next(), words.next(), words.next())
        {
            let class = match group {
                "cpKeyword" => "keyword",
                "cpBoolean" => "literal",
                _ => continue,
            };
            out.extend(words.map(|w| (w.to_string(), class)));
        }
    }
    out
}

/// The words of the `\_<\(?:a\|b\)\_>` rules of Emacs's grammar.
fn emacs_words() -> BTreeMap<String, &'static str> {
    let mut out = BTreeMap::new();
    for line in grammar::emacs().lines() {
        if !line
            .trim_start_matches([' ', '('])
            .starts_with("\"\\\\_<\\\\(?:")
            || line.contains("0[xX]")
        {
            continue;
        }
        let Some(body) = line
            .split("\\\\_<\\\\(?:")
            .nth(1)
            .and_then(|b| b.split("\\\\)\\\\_>").next())
        else {
            continue;
        };
        let class = if line.contains("keyword-face") {
            "keyword"
        } else {
            "literal"
        };
        out.extend(body.split("\\\\|").map(|w| (w.to_string(), class)));
    }
    out
}

fn sources(dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(std::fs::read_to_string(path).unwrap());
        }
    }
}

#[test]
fn the_generated_grammars_classify_words_as_the_lexer_does() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut texts = Vec::new();
    for dir in ["std", "examples"] {
        sources(&root.join(dir), &mut texts);
    }
    assert!(texts.len() > 20, "{} sources", texts.len());
    let (vim, emacs) = (vim_words(), emacs_words());
    let mut seen = 0;
    for text in &texts {
        for token in lex(text).tokens {
            let word = &text[token.span.start as usize..token.span.end as usize];
            if !word.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
                continue;
            }
            let lexed = if LITERAL_WORDS.contains(&word) {
                Some("literal")
            } else if token.kind == TokenKind::Ident || TokenKind::keyword(word).is_none() {
                None
            } else {
                Some("keyword")
            };
            seen += 1;
            assert_eq!(vim.get(word).copied(), lexed, "vim on `{word}`");
            assert_eq!(emacs.get(word).copied(), lexed, "emacs on `{word}`");
        }
    }
    assert!(seen > 1000, "{seen} words");
    for word in vim.keys().chain(emacs.keys()) {
        assert!(
            TokenKind::keyword(word).is_some() || LITERAL_WORDS.contains(&word.as_str()),
            "`{word}` is no keyword"
        );
    }
}

/// editors/words.json, which tree-sitter and JetBrains read, is the lexer's tables (`just gen`).
#[test]
fn the_editor_word_list_is_the_lexers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join("editors/words.json")).unwrap();
    let list = |key: &str| -> Vec<String> {
        let body = text
            .split(&format!("\"{key}\": ["))
            .nth(1)
            .unwrap()
            .split(']')
            .next()
            .unwrap();
        body.split(',')
            .map(|w| w.trim().trim_matches('"').to_string())
            .filter(|w| !w.is_empty())
            .collect()
    };
    assert_eq!(
        list("keywords"),
        crate::KEYWORDS,
        "editors/words.json is stale; run: just gen"
    );
    assert_eq!(
        list("literals"),
        LITERAL_WORDS,
        "editors/words.json is stale; run: just gen"
    );
}
