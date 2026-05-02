#![doc = include_str!("../README.md")]

use std::io::{self, Read};
use std::process;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, markdown_to_html, parse_document};
use objc2_app_kit::NSPasteboard;
use objc2_foundation::NSString;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn slurp_stdin() -> Result<String> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    Ok(String::from_utf8(bytes)?)
}

fn append_inline_text<'a>(node: &'a AstNode<'a>, out: &mut String) {
    for child in node.children() {
        match &child.data.borrow().value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(c) => out.push_str(&c.literal),
            NodeValue::SoftBreak => out.push(' '),
            NodeValue::LineBreak => out.push('\n'),
            NodeValue::HtmlInline(_) => {}
            _ => append_inline_text(child, out),
        }
    }
}

fn append_block_text<'a>(node: &'a AstNode<'a>, out: &mut String) {
    let mut first = true;
    for child in node.children() {
        let separator = if first { "" } else { "\n\n" };
        match &child.data.borrow().value {
            NodeValue::Paragraph | NodeValue::Heading(_) => {
                out.push_str(separator);
                append_inline_text(child, out);
                first = false;
            }
            NodeValue::BlockQuote | NodeValue::List(_) | NodeValue::Item(_) => {
                out.push_str(separator);
                append_block_text(child, out);
                first = false;
            }
            NodeValue::CodeBlock(cb) => {
                out.push_str(separator);
                out.push_str(cb.literal.trim_end_matches('\n'));
                first = false;
            }
            NodeValue::ThematicBreak => {
                out.push_str(separator);
                first = false;
            }
            _ => {}
        }
    }
}

fn render_plain(markdown: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &Options::default());
    let mut out = String::new();
    append_block_text(root, &mut out);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn render_html(markdown: &str) -> String {
    let mut options = Options::default();
    options.extension.autolink = true;
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    markdown_to_html(markdown, &options)
}

fn write_clipboard(html: &str, plain: &str) {
    let html_ns = NSString::from_str(html);
    let plain_ns = NSString::from_str(plain);
    let html_type = NSString::from_str("public.html");
    let plain_type = NSString::from_str("public.utf8-plain-text");

    let pb = NSPasteboard::generalPasteboard();
    pb.clearContents();
    pb.setString_forType(&html_ns, &html_type);
    pb.setString_forType(&plain_ns, &plain_type);
}

fn run() -> Result<()> {
    let markdown = slurp_stdin()?;
    let html = render_html(&markdown);
    let plain = render_plain(&markdown);
    write_clipboard(&html, &plain);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("mdrich: {err}");
        process::exit(1);
    }
}
