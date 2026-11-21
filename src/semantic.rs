//! Semantic tokens for an editor, as `parser-rs` encodes them. txt2 has
//! no types and functions: each kind of markup takes a standard token
//! type, and the editor maps them to markup scopes for txt2 (bold to
//! `markup.bold` ...):
//!
//! | markup | token type |
//! |---|---|
//! | markers (`**`, `\|`, `>`) | `operator` |
//! | block tags (`((кат`, `))`, `>>`, fences) | `keyword` |
//! | a block's title | `namespace` |
//! | heading | `class` |
//! | bold, italic, struck | `type`, `parameter`, `enumMember` |
//! | accents `!!` `++` `??` | `event`, `method`, `macro` |
//! | offtop | `comment` |
//! | label | `decorator` |
//! | code | `string` |
//! | link text, link | `function`, `regexp` |
//! | mention, search | `property` |
//! | icon | `enum` |
//! | quote | `interface` |
//! | list marker | `number` |
//! | progress | `typeParameter` |

use parser_rs::semantic::encode;
use parser_rs::{Modifiers, SemanticToken, SemanticType as T};

use crate::ast::TokenKind;

/// The token type of a kind of markup.
pub fn token_type(kind: TokenKind) -> T {
    match kind {
        TokenKind::Marker => T::Operator,
        TokenKind::Tag => T::Keyword,
        TokenKind::Title => T::Namespace,
        TokenKind::Heading => T::Class,
        TokenKind::Bold => T::Type,
        TokenKind::Italic => T::Parameter,
        TokenKind::Strike => T::EnumMember,
        TokenKind::Accent(1) => T::Event,
        TokenKind::Accent(2) => T::Method,
        TokenKind::Accent(_) => T::Macro,
        TokenKind::Offtop => T::Comment,
        TokenKind::Label => T::Decorator,
        TokenKind::Code => T::String,
        TokenKind::Link => T::Function,
        TokenKind::Url => T::Regexp,
        TokenKind::Mention => T::Property,
        TokenKind::Icon => T::Enum,
        TokenKind::Quote => T::Interface,
        TokenKind::ListMarker => T::Number,
        TokenKind::Progress => T::TypeParameter,
    }
}

/// Every token of `src`, unencoded.
pub fn semantic_tokens(src: &str) -> Vec<SemanticToken> {
    crate::parse(src)
        .tokens
        .iter()
        .map(|t| SemanticToken::new(t.line, t.start, t.end, token_type(t.kind), Modifiers::NONE))
        .collect()
}

/// [`semantic_tokens`] in the editor's wire format.
pub fn encoded_semantic_tokens(src: &str) -> Vec<u32> {
    encode(semantic_tokens(src), src)
}
