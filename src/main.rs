mod ast;
mod compiler;
mod lexer;
mod semantic;
mod traits;

use compiler::LolCompiler;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};
use traits::Compiler;

// Print the usage message and exit. The ! means this function never returns.
fn usage_error(msg: &str) -> ! {
    eprintln!("Usage error: {}", msg);
    eprintln!("Usage: lolcompiler <input.lol>");
    process::exit(1);
}

// Try to open the HTML file. If this fails, the saved file is still usable.
fn open_in_browser(path: &Path) {
    // Pick the open command for this operating system.
    let p = path.to_string_lossy().to_string();
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", &p]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&p).spawn()
    } else {
        Command::new("xdg-open").arg(&p).spawn()
    };
    if result.is_err() {
        eprintln!(
            "Note: could not open a browser automatically. Open {} manually.",
            p
        );
    }
}

// Read the file, compile it, then save and open the HTML.
fn main() {
    // We need exactly one filename.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage_error("expected exactly one argument (the .lol file to compile).");
    }

    // Allow .lol in either uppercase or lowercase.
    let input = Path::new(&args[1]);
    let is_lol = input
        .extension()
        .map(|e| e.to_string_lossy().eq_ignore_ascii_case("lol"))
        .unwrap_or(false);
    if !is_lol {
        usage_error(&format!("'{}' is not a .lol file.", args[1]));
    }

    let source = match fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => usage_error(&format!("could not read '{}': {}", args[1], e)),
    };

    // Turn the source into HTML.
    let mut compiler = LolCompiler::new();
    compiler.compile(&source);

    // Save next to the input file, with .html as the extension.
    let output = input.with_extension("html");
    if let Err(e) = fs::write(&output, &compiler.html) {
        eprintln!("Error: could not write '{}': {}", output.display(), e);
        process::exit(1);
    }
    println!("Compiled {} -> {}", input.display(), output.display());
    open_in_browser(&output);
}
