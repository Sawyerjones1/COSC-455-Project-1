/// The compiler steps. Stop and report an error if compilation fails.
pub trait Compiler {
    /// Compile the source text.
    fn compile(&mut self, source: &str);

    /// Ask the lexer for the next token.
    fn next_token(&mut self) -> String;

    /// Parse the program starting with the lolcode rule.
    fn parse(&mut self);

    /// Get the token we are currently looking at.
    fn current_token(&self) -> String;

    /// Set the current token.
    fn set_current_token(&mut self, tok: String);
}

/// Read the input one character at a time, following the state machine.
pub trait LexicalAnalyzer {
    /// Get the next character, or '\0' at the end of the input.
    fn get_char(&mut self) -> char;

    /// Add a character to the token we are building.
    fn add_char(&mut self, c: char);

    /// Check whether a string is a valid token.
    fn lookup(&self, s: &str) -> bool;
}

/// Recursive descent parser: one function for each grammar rule.
/// Return Ok(()) if it matches, or an error message if it doesn't.
pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self) -> Result<(), String>;
    fn parse_head(&mut self) -> Result<(), String>;
    fn parse_title(&mut self) -> Result<(), String>;
    fn parse_comment(&mut self) -> Result<(), String>;
    fn parse_body(&mut self) -> Result<(), String>;
    fn parse_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_text(&mut self) -> Result<(), String>;
    fn parse_variable_define(&mut self) -> Result<(), String>;
    fn parse_variable_use(&mut self) -> Result<(), String>;
    fn parse_bold(&mut self) -> Result<(), String>;
    fn parse_list(&mut self) -> Result<(), String>;
    fn parse_list_items(&mut self) -> Result<(), String>;
    fn parse_inner_list(&mut self) -> Result<(), String>;
    fn parse_link(&mut self) -> Result<(), String>;
    fn parse_newline(&mut self) -> Result<(), String>;
    fn parse_text(&mut self) -> Result<(), String>;
}
