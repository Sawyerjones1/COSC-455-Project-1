use crate::traits::LexicalAnalyzer;

// The tags this language accepts.
pub const ANNOTATIONS: [&str; 17] = [
    "#HAI",
    "#KBYE",
    "#OBTW",
    "#TLDR",
    "#MKAY",
    "#OIC",
    "#NEWLINE",
    "#IHAZ",
    "#ITIZ",
    "#MAEK HEAD",
    "#MAEK PARAGRAF",
    "#MAEK LIST",
    "#GIMMEH TITLE",
    "#GIMMEH BOLD",
    "#GIMMEH ITEM",
    "#GIMMEH LINX",
    "#GIMMEH VAR",
];

// Read characters using the start, tag, text, and error states from the DFA.
// Check each token before passing it to the parser.
pub struct Lexer {
    // Use chars so we don't have to index into UTF-8 bytes.
    chars: Vec<char>,
    // Where to read next.
    pos: usize,
    // Line and column numbers start at 1.
    line: usize,
    col: usize,
    // The token we're building.
    lexeme: String,
    // Remember where the token started, even if it spans multiple lines.
    pub token_line: usize,
    // The starting column for error messages.
    pub token_col: usize,
}

pub fn is_text_char(c: char) -> bool {
    // These are the characters allowed in regular text.
    c.is_ascii_alphanumeric() || matches!(c, ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/')
}

// Stop at whitespace, the next tag, or the end of the file.
fn ends_token(c: char) -> bool {
    c.is_whitespace() || c == '#' || c == '\0'
}

impl Lexer {
    // Start at the beginning of the source.
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            lexeme: String::new(),
            token_line: 1,
            token_col: 1,
        }
    }

    fn peek(&self) -> char {
        // Look ahead without moving. '\0' means we're at the end.
        *self.chars.get(self.pos).unwrap_or(&'\0')
    }

    // Read a character and add it to the current token.
    fn take(&mut self) {
        let c = self.get_char();
        self.add_char(c);
    }

    // Read one token. An empty string means we're done; Err means a bad token.
    pub fn lex(&mut self) -> Result<String, String> {
        self.lexeme.clear();

        // Start state: skip whitespace and save where this token begins.
        while self.peek().is_whitespace() {
            self.get_char();
        }
        self.token_line = self.line;
        self.token_col = self.col;

        let c = self.peek();
        if c == '\0' {
            return Ok(String::new());
        }

        if c == '#' {
            // Tag state: read the letters after '#'.
            self.take();
            while self.peek().is_ascii_alphabetic() {
                self.take();
            }
            let first = self.lexeme.to_uppercase();
            // #MAEK and #GIMMEH have two words. Use one space between them,
            // even if the source uses extra spaces, tabs, or newlines.
            if (first == "#MAEK" || first == "#GIMMEH") && self.peek().is_whitespace() {
                while self.peek().is_whitespace() {
                    self.get_char();
                }
                self.add_char(' ');
                while self.peek().is_ascii_alphabetic() {
                    self.take();
                }
            }
        } else if is_text_char(c) {
            // Text state: read until we hit something that isn't a text character.
            while is_text_char(self.peek()) {
                self.take();
            }
        } else {
            // Error state: this character can't start a token.
            self.take();
            return Err(self.error());
        }

        // Read the rest of a bad token so the error shows all of it, like abc$def.
        if !ends_token(self.peek()) {
            while !ends_token(self.peek()) {
                self.take();
            }
            return Err(self.error());
        }
        if !self.lookup(&self.lexeme) {
            // Even a tag made of valid letters can be unknown, like #HEAD.
            return Err(self.error());
        }
        // Uppercase the tags, but keep regular text as it was written.
        if c == '#' {
            Ok(self.lexeme.to_uppercase())
        } else {
            Ok(self.lexeme.clone())
        }
    }

    // Point to where the bad token started.
    fn error(&self) -> String {
        format!(
            "Lexical error at line {}, column {}: '{}' is not a valid LOLCODE token.",
            self.token_line,
            self.token_col,
            self.lexeme.trim()
        )
    }
}

impl LexicalAnalyzer for Lexer {
    // Read the next character and update our position.
    // At the end, keep returning '\0' without moving.
    fn get_char(&mut self) -> char {
        if self.pos >= self.chars.len() {
            return '\0';
        }
        let c = self.chars[self.pos];
        self.pos += 1;
        // Keep track of the line and column for errors.
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    // Add to the token. This doesn't move forward in the input.
    fn add_char(&mut self, c: char) {
        self.lexeme.push(c);
    }

    // Check tags against the list, ignoring case.
    // Regular text has to be nonempty and use allowed characters.
    fn lookup(&self, s: &str) -> bool {
        if s.starts_with('#') {
            ANNOTATIONS.contains(&s.to_uppercase().as_str())
        } else {
            !s.is_empty() && s.chars().all(is_text_char)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Read all the tokens for a test, stopping if there's an error.
    fn all_tokens(src: &str) -> Result<Vec<String>, String> {
        let mut lx = Lexer::new(src);
        let mut out = Vec::new();
        loop {
            let t = lx.lex()?;
            if t.is_empty() {
                return Ok(out);
            }
            out.push(t);
        }
    }

    #[test]
    // Check valid tags, mixed case, regular text, and an unknown tag.
    fn lookup_accepts_annotations_any_case() {
        let lx = Lexer::new("");
        assert_eq!(lx.lookup("#HAI"), true);
        assert_eq!(lx.lookup("#hai"), true);
        assert_eq!(lx.lookup("#Maek Paragraf"), true);
        assert_eq!(lx.lookup("#HEY"), false);
        assert_eq!(lx.lookup("Simpson"), true);
    }

    #[test]
    // Extra spaces or tabs should still give us one two-word tag.
    fn two_word_tags_are_one_token() {
        assert_eq!(
            all_tokens("#maek   head #GIMMEH\tTITLE Hi #OIC").unwrap(),
            vec!["#MAEK HEAD", "#GIMMEH TITLE", "Hi", "#OIC"]
        );
    }

    #[test]
    // These look like tags, but they aren't in the language.
    fn unknown_tag_is_lexical_error() {
        assert!(all_tokens("#HAI #HEAD #KBYE").is_err());
        assert!(all_tokens("#MAEK BOLD").is_err());
    }

    #[test]
    // Check reading characters, reaching EOF, and adding to the token.
    fn get_char_and_add_char() {
        let mut lx = Lexer::new("ab");
        assert_eq!(lx.get_char(), 'a');
        assert_eq!(lx.get_char(), 'b');
        assert_eq!(lx.get_char(), '\0');
        lx.add_char('x');
        assert_eq!(lx.lexeme, "x");
    }
}
