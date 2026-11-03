use crate::ast::{is_punct_word, Node};
use std::collections::HashMap;

// Walk the tree, look up variables, and build the HTML.
struct Generator {
    // Each block gets a scope. The last one is the innermost block.
    scopes: Vec<HashMap<String, String>>,
    // HTML built so far. Only return it if there are no errors.
    out: String,
}

pub fn generate(tree: &Node) -> Result<String, String> {
    // Start with no scopes or output, then walk the tree.
    let mut g = Generator {
        scopes: Vec::new(),
        out: String::new(),
    };
    g.node(tree)?;
    Ok(g.out)
}

impl Generator {
    // Remove leftover spaces before punctuation, closing tags, or line breaks.
    fn trim_space(&mut self) {
        while self.out.ends_with(' ') {
            self.out.pop();
        }
    }

    fn inline(&mut self, s: &str) {
        // Attach punctuation to the previous text. Add a space for whatever comes next;
        // trim_space takes it back off when needed.
        if is_punct_word(s.split(' ').next().unwrap_or("")) {
            self.trim_space();
        }
        self.out.push_str(s);
        self.out.push(' ');
    }

    // Give this block its own scope and write its HTML.
    // Inline blocks end with a space; other blocks end with a newline.
    fn block(
        &mut self,
        open: &str,
        close: &str,
        kids: &[Node],
        inline: bool,
    ) -> Result<(), String> {
        // Add a scope for the children of this block.
        self.scopes.push(HashMap::new());
        self.out.push_str(open);
        for k in kids {
            self.node(k)?;
        }
        self.trim_space();
        // For multiline tags, put the closing tag on its own line.
        if open.ends_with('\n') && !self.out.ends_with('\n') {
            self.out.push('\n');
        }
        self.out.push_str(close);
        if inline {
            self.out.push(' ');
        } else {
            self.out.push('\n');
        }
        // We're done with this block, so its variables go out of scope.
        self.scopes.pop();
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<String> {
        // Check the closest scope first, so a local variable can hide an outer one.
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    fn node(&mut self, n: &Node) -> Result<(), String> {
        // Process nodes in order so variables are defined before we use them.
        // An undefined variable stops HTML generation.
        match n {
            Node::Document(k) => self.block("<html>\n", "</html>", k, false)?,
            Node::Head(k) => self.block("<head>\n", "</head>", k, false)?,
            Node::Title(t) => self.out.push_str(&format!("<title>{}</title>\n", t)),
            Node::Comment(t) => self.out.push_str(&format!("<!-- {} -->\n", t)),
            Node::Paragraph(k) => self.block("<p>", "</p>", k, false)?,
            Node::Bold(k) => self.block("<b>", "</b>", k, true)?,
            Node::List(k) => self.block("<ul>\n", "</ul>", k, false)?,
            Node::Item(k) => self.block("<li>", "</li>", k, false)?,
            Node::Link(a) => self.inline(&format!("<a href=\"{}\">{}</a>", a, a)),
            Node::Newline => {
                self.trim_space();
                self.out.push_str("<br>\n");
            }
            Node::VarDef { name, value } => {
                // Remember the value here. A definition doesn't add any HTML.
                self.scopes
                    .last_mut()
                    .expect("no active scope")
                    .insert(name.clone(), value.clone());
            }
            Node::VarUse { name, line } => match self.lookup(name) {
                Some(v) => self.inline(&v),
                None => {
                    return Err(format!(
                        "Static semantic error at line {}: variable '{}' is used but not defined in this scope.",
                        line, name
                    ))
                }
            },
            Node::Text(t) => self.inline(t),
        }
        Ok(())
    }
}
