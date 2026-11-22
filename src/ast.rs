//! What a txt2 text is made of.

use parser_rs::ParseError;

/// A parsed text.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub blocks: Vec<Block>,
    /// What's wrong: unclosed blocks, closings without an opening.
    pub errors: Vec<ParseError>,
    /// What to highlight, in no order.
    pub tokens: Vec<Token>,
}

/// A piece of text to highlight: line, columns `start..end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

/// What a token is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// Markup characters: `**`, `>`, `|`, `---` ...
    Marker,
    /// A block's tag: `((кат`, `))`, `>>`, `<<`, a code fence.
    Tag,
    /// A block's title: a spoiler's, a poll's, a tile's.
    Title,
    Heading,
    Bold,
    Italic,
    Strike,
    /// `!!`, `++`, `??`: 1, 2, 3.
    Accent(u8),
    Offtop,
    Label,
    /// Inline code and code blocks: text as it is.
    Code,
    /// A link's text.
    Link,
    Url,
    /// `@user`, `@p/project`, `w:search`.
    Mention,
    Icon,
    Quote,
    ListMarker,
    Progress,
}

/// A block, from its first line to its last (its closing line
/// included).
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub line: usize,
    pub end: usize,
    pub kind: BlockKind,
}

/// The text of `inlines` without its markup: a title, a caption.
pub fn plain(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for i in inlines {
        match i {
            Inline::Text(t) | Inline::Code(t) => out += t,
            Inline::Bold(x) | Inline::Italic(x) | Inline::Strike(x) | Inline::Accent(_, x) | Inline::Offtop(x) | Inline::Label(_, x) => {
                out += &plain(x)
            }
            Inline::Link { text, .. } => out += &plain(text),
            Inline::User(n) | Inline::Project(n) => out += n,
            Inline::Search(_, t) => out += &t.replace('+', " "),
            Inline::Icon(..) => {}
        }
    }
    out
}

/// How an image or a block is aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    /// Lines of text, each its own line.
    Paragraph(Vec<Vec<Inline>>),
    Heading { level: u8, text: Vec<Inline> },
    /// `---`
    Rule,
    Quote(Vec<Block>),
    Code { lang: Option<String>, text: String },
    /// `{{` ... `}}`: text without formatting, its lines kept.
    Raw(String),
    /// `>>` ... `<<`, `((кат` ... `))`.
    Spoiler { title: Option<Vec<Inline>>, blocks: Vec<Block> },
    /// `((центр`, `((вправо`.
    Aligned { align: Align, blocks: Vec<Block> },
    /// `((макет`: a row of `Column`s.
    Layout(Vec<Block>),
    /// `((блок33`, `((блок50`, `((блок66`: its width in percent.
    Column { width: u8, blocks: Vec<Block> },
    List(List),
    Table(Table),
    /// A progress bar and the text after it.
    Progress(Progress, Vec<Inline>),
    /// A link on a line of its own: an image, a video, a sound, a gist.
    Media(Embed),
    /// Images on lines one after another, none aligned: two or more.
    Gallery(Vec<Embed>),
    Poll(Poll),
    Tile(Tile),
}

/// What a link on a line of its own shows.
#[derive(Debug, Clone, PartialEq)]
pub struct Embed {
    pub kind: Media,
    pub url: String,
    /// `<-`, `->`, `<>` before it.
    pub align: Option<Align>,
    /// After ` - `.
    pub caption: Option<Vec<Inline>>,
    /// `(url)` after it: an image that is a link.
    pub link: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Media {
    Image,
    Video,
    Audio,
    Gist,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub ordered: bool,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// As written: `-`, `*`, `1.`, `2.1.`, `3)`.
    pub marker: String,
    pub text: Vec<Inline>,
    /// Lines under it, indented more: more text and nested lists.
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// How many of the first rows are the header (before `---`).
    pub header: usize,
    pub rows: Vec<Vec<Cell>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Text { align: Option<Align>, text: Vec<Inline> },
    /// `<`: joined with the cell on its left.
    Left,
    /// `^`: joined with the cell above.
    Up,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Progress {
    /// `[ 5% ]`
    Percent(f64),
    /// `[ 2 / 5 ]`
    Of(f64, f64),
    /// `[ 4 / 3 - 19 ]`: current, minimum, maximum.
    Range(f64, f64, f64),
    /// `[ 28.11.2023 / 27.11.2025 ]`: from, to.
    Dates(String, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Poll {
    pub title: String,
    pub options: Vec<Vec<Inline>>,
    /// `мультивыбор N`: how many to choose; `Some(None)`: half of them.
    pub multiple: Option<Option<u32>>,
    /// `закрывается dd.mm.yyyy`
    pub closes: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pub title: String,
    pub url: String,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
}

/// Text inside a line.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strike(Vec<Inline>),
    /// 1 `!!`, 2 `++`, 3 `??`.
    Accent(u8, Vec<Inline>),
    Offtop(Vec<Inline>),
    /// `[[[x]]]` (0), `[!!x!!]` (1), `[++x++]` (2), `[??x??]` (3).
    Label(u8, Vec<Inline>),
    Code(String),
    Link { text: Vec<Inline>, url: String },
    /// `@name` (spaces as `+`), `~name`, `~~name~~`.
    User(String),
    /// `@p/name`
    Project(String),
    /// `w:term` ...: the engine's letter, the term.
    Search(char, String),
    /// `/иконка/name/2x`: the name, the size (1 to 5).
    Icon(String, u8),
}
