//! txt2, the markup of XGM (xgm.guru): a parser that never fails (what
//! isn't markup is text), an HTML renderer for a preview, and semantic
//! tokens for an editor.
//!
//! ```
//! let doc = txt2_rs::parse("== Title ==\n*bold* and ++green++\n");
//! assert_eq!(doc.blocks.len(), 2);
//! let html = txt2_rs::html::render(&doc);
//! assert!(html.contains("<b>bold</b>"));
//! ```
//!
//! The syntax is in the README. Lines and columns count from 1, columns
//! in characters.

pub mod ast;
mod block;
pub mod html;
mod inline;
pub mod semantic;

pub use ast::*;
pub use parser_rs::ParseError;

/// Parses `text`: its blocks, what's wrong (an unclosed block ...), and
/// the tokens to highlight it with.
pub fn parse(text: &str) -> Document {
    block::parse(text)
}

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
