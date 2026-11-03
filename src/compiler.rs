use crate::ast::{join_words, Node};
use crate::lexer::Lexer;
use crate::semantic;
use crate::traits::{Compiler, SyntaxAnalyzer};
use std::process;

// Run the lexer and parser, then generate HTML from the tree.
// The tree keeps track of blocks so we can check variable scopes later.
pub struct LolCompiler {
    lexer: Lexer,
    current: String,
    nodes: Vec<Node>,
    pub tree: Option<Node>,
    // The HTML that main will save.
    pub html: String,
}

impl LolCompiler {
    // Set up an empty compiler. compile loads the source.
    pub fn new() -> Self {
        LolCompiler {
            lexer: Lexer::new(""),
            current: String::new(),
            nodes: Vec::new(),
            tree: None,
            html: String::new(),
        }
    }

    // Show 'end of file' in errors instead of an empty string.
    fn describe(tok: &str) -> String {
        if tok.is_empty() {
            "end of file".to_string()
        } else {
            format!("'{}'", tok)
        }
    }

    // Show what we expected and what we actually found.
    fn syntax_error(&self, expected: &str) -> String {
        format!(
            "Syntax error at line {}: expected {} but found {}.",
            self.lexer.token_line,
            expected,
            Self::describe(&self.current_token())
        )
    }

    // Move on to the next token.
    fn advance(&mut self) {
        let t = self.next_token();
        self.set_current_token(t);
    }

    fn expect(&mut self, tok: &str) -> Result<(), String> {
        // Move past this token if it matches, or report an error.
        if self.current == tok {
            self.advance();
            Ok(())
        } else {
            Err(self.syntax_error(&format!("'{}'", tok)))
        }
    }

    // The lexer already checked the characters. A word is anything left
    // that isn't a tag or the end of the file.
    fn is_word(&self) -> bool {
        !self.current.is_empty() && !self.current.starts_with('#')
    }

    // Read one word. The caller tells us what to call it in an error message.
    fn take_word(&mut self, what: &str) -> Result<String, String> {
        if self.is_word() {
            let w = self.current.clone();
            self.advance();
            Ok(w)
        } else {
            Err(self.syntax_error(what))
        }
    }

    fn read_text(&mut self) -> Result<String, String> {
        // Read at least one word, stopping at the next tag.
        // join_words fixes the spaces around punctuation.
        let mut words = vec![self.take_word("text")?];
        while self.is_word() {
            words.push(self.current.clone());
            self.advance();
        }
        Ok(join_words(&words))
    }
}

impl Compiler for LolCompiler {
    // Run each step. If one fails, stop before main saves the HTML.
    fn compile(&mut self, source: &str) {
        // Clear anything left from a previous compilation.
        self.lexer = Lexer::new(source);
        self.nodes.clear();
        self.tree = None;
        self.html.clear();

        self.advance();
        self.parse();

        // Check the variables and build HTML from the tree.
        let tree = self.tree.as_ref().expect("no parse tree");
        match semantic::generate(tree) {
            Ok(html) => self.html = html,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }

    // Get the next token or stop on a lexer error.
    // advance is what actually updates the current token.
    fn next_token(&mut self) -> String {
        match self.lexer.lex() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }

    // Parse the program and put its nodes under one document node.
    // Print an error and stop if parsing fails.
    fn parse(&mut self) {
        match self.parse_lolcode() {
            Ok(()) => {
                let children = std::mem::take(&mut self.nodes);
                self.tree = Some(Node::Document(children));
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }

    // Get a copy of the current token without moving forward.
    fn current_token(&self) -> String {
        self.current.clone()
    }

    // Only change the stored token; this doesn't read from the lexer.
    fn set_current_token(&mut self, tok: String) {
        self.current = tok;
    }
}

impl SyntaxAnalyzer for LolCompiler {
    fn parse_lolcode(&mut self) -> Result<(), String> {
        // Parse the sections in order. Nothing should come after #KBYE.
        self.expect("#HAI")?;
        self.parse_comment()?;
        self.parse_head()?;
        self.parse_variable_define()?;
        self.parse_body()?;
        self.expect("#KBYE")?;
        if !self.current.is_empty() {
            return Err(self.syntax_error("end of file after '#KBYE'"));
        }
        Ok(())
    }

    // Skip the head if it isn't there. If it is, it needs a title and #MKAY.
    fn parse_head(&mut self) -> Result<(), String> {
        if self.current == "#MAEK HEAD" {
            self.advance();
            let start = self.nodes.len();
            self.parse_comment()?;
            self.parse_title()?;
            self.expect("#MKAY")?;
            // Group just the nodes added inside this head.
            // The other block parsers do the same thing.
            let kids = self.nodes.split_off(start);
            self.nodes.push(Node::Head(kids));
            self.parse_comment()?;
        }
        Ok(())
    }

    // A title needs some text and #OIC. This rule is only called inside a head.
    fn parse_title(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH TITLE")?;
        let t = self.read_text()?;
        self.expect("#OIC")?;
        self.nodes.push(Node::Title(t));
        self.parse_comment()
    }

    // Read any comments here. An empty comment still needs #TLDR.
    fn parse_comment(&mut self) -> Result<(), String> {
        while self.current == "#OBTW" {
            self.advance();
            let t = if self.is_word() {
                self.read_text()?
            } else {
                String::new()
            };
            self.expect("#TLDR")?;
            self.nodes.push(Node::Comment(t));
        }
        Ok(())
    }

    fn parse_body(&mut self) -> Result<(), String> {
        // Use the current token to pick which rule to try.
        // Stop on anything else so the caller can check the closing tag.
        loop {
            match self.current.as_str() {
                "#MAEK PARAGRAF" => self.parse_paragraph()?,
                "#GIMMEH BOLD" => self.parse_bold()?,
                "#MAEK LIST" => self.parse_list()?,
                "#GIMMEH LINX" => self.parse_link()?,
                "#NEWLINE" => self.parse_newline()?,
                "#GIMMEH VAR" => self.parse_variable_use()?,
                _ if self.is_word() => self.parse_text()?,
                _ => return Ok(()),
            }
            self.parse_comment()?;
        }
    }

    fn parse_paragraph(&mut self) -> Result<(), String> {
        // Save the start so we can group this paragraph's nodes.
        self.expect("#MAEK PARAGRAF")?;
        let start = self.nodes.len();
        self.parse_comment()?;
        self.parse_variable_define()?;
        self.parse_inner_paragraph()?;
        self.expect("#MKAY")?;
        let kids = self.nodes.split_off(start);
        self.nodes.push(Node::Paragraph(kids));
        Ok(())
    }

    // Read paragraph content. Paragraphs can't contain other paragraphs.
    // Let parse_paragraph check the closing tag.
    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        loop {
            match self.current.as_str() {
                "#GIMMEH BOLD" => self.parse_bold()?,
                "#MAEK LIST" => self.parse_list()?,
                "#GIMMEH LINX" => self.parse_link()?,
                "#NEWLINE" => self.parse_newline()?,
                "#GIMMEH VAR" => self.parse_variable_use()?,
                _ if self.is_word() => self.parse_text()?,
                _ => return Ok(()),
            }
            self.parse_comment()?;
        }
    }

    fn parse_inner_text(&mut self) -> Result<(), String> {
        // Bold content can be regular text or variable uses.
        loop {
            match self.current.as_str() {
                "#GIMMEH VAR" => self.parse_variable_use()?,
                _ if self.is_word() => self.parse_text()?,
                _ => return Ok(()),
            }
        }
    }

    // Read the definitions at the start of a block. Each needs a name and value.
    // Save them in the tree; we'll sort out scopes when generating HTML.
    fn parse_variable_define(&mut self) -> Result<(), String> {
        while self.current == "#IHAZ" {
            self.advance();
            let name = self.take_word("a variable name")?;
            self.expect("#ITIZ")?;
            let value = self.read_text()?;
            self.expect("#MKAY")?;
            self.nodes.push(Node::VarDef { name, value });
            self.parse_comment()?;
        }
        Ok(())
    }

    // Read the variable name and #OIC. We'll check if it's defined later.
    fn parse_variable_use(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH VAR")?;
        // Keep the line number in case this variable isn't defined.
        let line = self.lexer.token_line;
        let name = self.take_word("a variable name")?;
        self.expect("#OIC")?;
        self.nodes.push(Node::VarUse { name, line });
        Ok(())
    }

    // Group the definitions and text into a bold node.
    // Bold needs its own #OIC, even if a variable use just had one.
    fn parse_bold(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH BOLD")?;
        let start = self.nodes.len();
        self.parse_variable_define()?;
        self.parse_inner_text()?;
        self.expect("#OIC")?;
        let kids = self.nodes.split_off(start);
        self.nodes.push(Node::Bold(kids));
        Ok(())
    }

    // Keep the list's definitions with its items so all items can use them.
    fn parse_list(&mut self) -> Result<(), String> {
        self.expect("#MAEK LIST")?;
        let start = self.nodes.len();
        self.parse_variable_define()?;
        self.parse_list_items()?;
        self.expect("#MKAY")?;
        let kids = self.nodes.split_off(start);
        self.nodes.push(Node::List(kids));
        Ok(())
    }

    fn parse_list_items(&mut self) -> Result<(), String> {
        // A list needs at least one item. Each item ends with its own #OIC.
        if self.current != "#GIMMEH ITEM" {
            return Err(self.syntax_error("'#GIMMEH ITEM'"));
        }
        while self.current == "#GIMMEH ITEM" {
            self.advance();
            let start = self.nodes.len();
            self.parse_variable_define()?;
            self.parse_inner_list()?;
            self.expect("#OIC")?;
            let kids = self.nodes.split_off(start);
            self.nodes.push(Node::Item(kids));
            self.parse_comment()?;
        }
        Ok(())
    }

    // Items can contain bold, variables, and text, but no lists or paragraphs.
    // parse_list_items handles the closing tag.
    fn parse_inner_list(&mut self) -> Result<(), String> {
        loop {
            match self.current.as_str() {
                "#GIMMEH BOLD" => self.parse_bold()?,
                "#GIMMEH VAR" => self.parse_variable_use()?,
                _ if self.is_word() => self.parse_text()?,
                _ => return Ok(()),
            }
        }
    }

    // Read one address with no spaces. It becomes both the link and its label.
    fn parse_link(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH LINX")?;
        let address = self.take_word("a link address")?;
        self.expect("#OIC")?;
        self.nodes.push(Node::Link(address));
        Ok(())
    }

    // #NEWLINE adds a line break; a newline in the source doesn't.
    fn parse_newline(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")?;
        self.nodes.push(Node::Newline);
        Ok(())
    }

    // Keep consecutive words together in one text node.
    fn parse_text(&mut self) -> Result<(), String> {
        let t = self.read_text()?;
        self.nodes.push(Node::Text(t));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(src: &str) -> Result<(), String> {
        // Call the parsing rule directly so a syntax error won't exit the tests.
        let mut c = LolCompiler::new();
        c.lexer = Lexer::new(src);
        c.advance();
        c.parse_lolcode()
    }

    #[test]
    // Try a document with a head, title, and body text.
    fn valid_document_parses() {
        assert_eq!(
            parse_ok("#HAI #MAEK HEAD #GIMMEH TITLE T #OIC #MKAY Hi #KBYE"),
            Ok(())
        );
    }

    #[test]
    // A title is valid inside a head, but not here.
    fn title_outside_head_is_syntax_error() {
        assert!(parse_ok("#HAI #GIMMEH TITLE T #OIC #KBYE").is_err());
    }

    #[test]
    // Make sure we reject a paragraph inside another paragraph.
    fn nested_paragraph_is_syntax_error() {
        assert!(parse_ok("#HAI #MAEK PARAGRAF #MAEK PARAGRAF x #MKAY #MKAY #KBYE").is_err());
    }

    #[test]
    // Check that we get back the token we set.
    fn current_token_round_trip() {
        let mut c = LolCompiler::new();
        c.set_current_token("#HAI".to_string());
        assert_eq!(c.current_token(), "#HAI");
    }

    #[test]
    fn valid_test_files_generate_html() {
        for (name, expected) in [
            ("Test2", "<title>Test 2</title>"),
            ("Test3", "<b>test case 3</b>"),
            ("Test6", "is 42</p>"),
            ("Test8", "really happiness</p>"),
            ("Test9", "really happiness</p>"),
            ("Test10", "<a href=\"https://www.youtube.com/watch?v=aboZctrHfK8\">"),
            ("MyTest1", "<li>Maggie Simpson</li>"),
            ("MyTest2", "<li>Homer Simpson</li>"),
            ("MyTest3", "Now my name is Josh again."),
        ] {
            let source = std::fs::read_to_string(format!("test/{name}.lol")).unwrap();
            let mut compiler = LolCompiler::new();
            compiler.compile(&source);
            assert!(compiler.html.starts_with("<html>"), "{name}");
            assert!(compiler.html.trim_end().ends_with("</html>"), "{name}");
            assert!(compiler.html.contains(expected), "{name}");
        }
    }

    #[test]
    fn paragraph_variable_does_not_replace_outer_variable() {
        let source = std::fs::read_to_string("test/MyTest3.lol").unwrap();
        let mut compiler = LolCompiler::new();
        compiler.compile(&source);
        assert!(compiler.html.contains("Hi, my name is Josh."));
        assert!(compiler.html.contains("Inside the paragraph block, my name is Jon"));
        assert!(compiler.html.contains("Now my name is Josh again."));
    }

    #[test]
    fn invalid_test_files_have_syntax_errors() {
        for name in ["Test5", "MyTest5", "MyTest6"] {
            let source = std::fs::read_to_string(format!("test/{name}.lol")).unwrap();
            assert!(parse_ok(&source).is_err(), "{name}");
        }
    }

    #[test]
    fn undefined_variables_are_rejected() {
        for name in ["Test7", "MyTest4"] {
            let source = std::fs::read_to_string(format!("test/{name}.lol")).unwrap();
            let mut compiler = LolCompiler::new();
            compiler.lexer = Lexer::new(&source);
            compiler.advance();
            compiler.parse_lolcode().unwrap();
            let tree = Node::Document(compiler.nodes);
            let error = semantic::generate(&tree).unwrap_err();
            assert!(error.contains("Static semantic error"), "{name}");
        }
    }
}
