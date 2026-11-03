
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Document(Vec<Node>),
    Comment(String),
    Head(Vec<Node>),
    Title(String),
    Paragraph(Vec<Node>),
    Bold(Vec<Node>),
    List(Vec<Node>),
    Item(Vec<Node>),
    Link(String),
    Newline,
    VarDef { name: String, value: String },
    VarUse { name: String, line: usize },
    Text(String),
}

// Checks if the whole string is punctuation.
pub fn is_punct_word(w: &str) -> bool {
    // These symbols go right after the previous word.
    w.chars()
        .all(|c| matches!(c, '.' | ',' | '!' | '?' | ':' | '%'))
}

// Put the words back into one string.
pub fn join_words(words: &[String]) -> String {
    // Add spaces between words, but leave punctuation attached.
    let mut out = String::new();
    for w in words {
        if !out.is_empty() && !is_punct_word(w) {
            out.push(' ');
        }
        out.push_str(w);
    }
    out
}
