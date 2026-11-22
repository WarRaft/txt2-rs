# txt2-rs

txt2 is the markup of [XGM](https://xgm.guru), the Warcraft III modding
site: its articles, resource pages and comments are written in it. This
crate reads it: a parser that never fails, an HTML renderer for a preview
and semantic tokens for an editor. The [wc3-rs](../wc3-rs) editor
highlights `.txt2` files with it and shows them beside the text, as
Markdown's preview does; [wc3-mcp](../wc3-mcp) gives it to models.

[`README.txt2`](README.txt2) is this document's format part written in
txt2 itself: it uses every construct, and the tests require it to parse
without errors.

```rust
let doc = txt2_rs::parse("== Title ==\n*bold* and ++green++\n");
let html = txt2_rs::html::render(&doc);        // a fragment, `data-line` on each block
let page = txt2_rs::html::page(&doc, "title");  // a whole page, with html::STYLE
let tokens = txt2_rs::semantic::encoded_semantic_tokens("**b**\n");
```

```sh
txt2-rs article.txt2 > article.html   # a page to open in a browser
txt2-rs --check article.txt2          # what's wrong, file:line:col: message
```

## The format

What follows is the format as this crate reads it: from the site's own
documentation, and from articles written in it where the two differ
(articles use single markers, `?byte?`, `+0+`, the documentation only
hints at). The site's keywords are Russian; each has an English twin,
and both are read the same.

A text is lines. A line break is kept: two lines are two lines on the
page, not one paragraph. Blank lines separate blocks. What isn't markup
is text, and a marker without its closing one stays as it's written.

### Inline

| markup | is |
|---|---|
| `*word*`, `**several words**` | bold |
| `_word_`, `__several words__` | italic |
| `-word-`, `--several words--` | struck through |
| `!word!`, `!!several words!!` | accent 1 (red) |
| `+word+`, `++several words++` | accent 2 (green) |
| `?word?`, `??several words??` | accent 3 (blue) |
| `// aside //`, `// to the end of the line` | offtop: small and muted |
| `[[[label]]]`, `[!!label!!]`, `[++label++]`, `[??label??]` | a label: plain, or in an accent's color |
| `{{text}}` | text as it is, no markup: inline code |
| `https://…` | a link, shown without its scheme |
| `__text__ (url)`, `_word_ (url)`, `Word (url)`, `[text](url)` | a link with text |
| `@name` (`+` for a space), `~name`, `~~two words~~` | a user |
| `@p/name` | a project |
| `w:term`, `wr:term` (`в:term`), `g:term`, `y:term` (`я:term`), `s:term` | a search: English Wikipedia, Russian Wikipedia, Google, Yandex, the site |
| `/icon/name`, `/icon/name/3x` (`/иконка/…`) | a FontAwesome 6 icon, 1x to 5x |

**Single and double markers.** A single marker wraps one word: it opens
after something that isn't a letter or digit (or at the line's start),
the next character isn't a space, and it closes at the next same marker
before anything that is a letter or digit, with no space between. So
`C++`, `snake_case`, `xgm-team`, `a - b` and `5 * 3` stay text. A double
marker may wrap several words: the character after it isn't a space, and
it closes at the next double marker (a space may come before that one:
`!!Note: !!`). Markers nest: `**bold __and italic__**`.

**Links.** The schemes are `http`, `https`, `ftp` (after `://`) and
`mailto`, `callto`, `skype`, `sip`, `sips`, `tg`, `steam`, `tel`,
`bitcoin`, `facetime` (after `:`). A bare link ends at a space; a
punctuation mark after it, or a `)` it didn't open, isn't part of it.
`Word (url)` makes the word before the parentheses the link's text.

### Blocks

A block starts a line (spaces before it are allowed). Blocks that open
run to their closing line, which is alone on its line; blocks inside
them are read as anywhere else.

| markup | is |
|---|---|
| `# Title` … `###### Title` | a heading, level the count of `#` (a space after them) |
| `== Title ==`, `===Title` | a heading, level the count of `=`; `=`s at the end are dropped |
| `---` (one or more `-`) | a horizontal rule |
| `> quote`, `> > nested` | a quote; `>` alone is an empty line in it |
| `((quote` … `))` (`((цитата`) | a quote of several lines |
| `>> title` … `<<`, `>>` … `<<` | a spoiler: hidden until clicked; without a title, «Show» |
| `((cut title` … `))`, `((spoiler title` (`((кат`) | a spoiler |
| `((center` … `))` (`((центр`), `((right` … `))` (`((вправо`) | centered, right-aligned |
| `((layout` … `))` (`((макет`) | a row of columns: `((block33`, `((block50`, `((block66` … `))` (`((блок33` …) inside it, 33, 50 or 66% wide |
| ```` ```lang ```` … ```` ``` ```` | code; the language is optional |
| `""` … `""` | code |
| `((code lang` … `))` (`((код`) | code |
| `{{` … `}}` | text as it is, its lines kept |
| `- item`, `* item` | a list |
| `1. item`, `1) item`, `2.1. item` | a numbered list |
| lines with `\|` (or tabs), after a blank line | a table |
| `[ … ]` and text | a progress bar |
| a link alone on its line | an image, a video, a sound or a gist |
| `((poll title` … `))` (`((опрос`) | a poll |
| `((tile title` … `))` (`((плитка`) | a navigation tile |

**Code.** Its lines are kept as they are, with no markup. A fence closes
at a line that is exactly the fence: ```` ``` ```` with a space after it
is a line of code. The languages the site highlights are `perl`, `php`,
`css`, `xml`, `html`, `javascript`, `java`, `cpp`, `sql`, `jass`, `lua`,
`delphi`, `dos`.

**Lists.** The items of a list share an indent (spaces or tabs, any
number). Lines under an item indented more than its marker belong to it:
more text, or a nested list. `-` and `*` are the same; a numbered marker
is digits with dots between (`2.1.`) and a `.` or `)` after, then a
space. A blank line ends a list.

**Tables.** A table starts after a blank line (or at a block's start)
with a line holding `|`; a line with tabs and no `|` is a row split at
its tabs, as pasted from a spreadsheet. Each line is a row; a row ends
the table when it has no `|`. Cells are trimmed. A line of `-`s after
the first rows makes them the header. A cell that is:

- `<` joins the cell on its left (spans one more column);
- `^` joins the cell above (spans one more row);
- `>>> text` is right-aligned, `<> text` centered.

A `|` inside `{{ }}` doesn't split: `{{|}}` is a `|` in a cell. Rows may
have different numbers of cells: the table is as wide as its widest row.

**Progress bars.** `[ 5% ]` a share; `[ 2 / 5 ]` a count of a total;
`[ 4 / 3 - 19 ]` a value between a minimum and a maximum;
`[ 28.11.2023 / 27.11.2025 ]` a countdown between two dates (Moscow
time). Text after the bar is its caption.

**Media.** A link alone on its line embeds:

- an image: `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.avif`, `.bmp`,
  `.svg` (also with `?…` after). `<- `, `-> `, `<> ` before it put it on
  the left or the right of the text, or in the center; ` - caption`
  after it adds a caption; ` (url)` after it makes it a link there. Two
  or more images on lines one after another, none aligned, are a
  gallery; a blank line between them keeps them apart;
- a video: YouTube, VK, Rutube, PLVideo, Twitch, Vimeo, Dailymotion,
  Bilibili pages, or `.mp4`, `.webm`, `.ogg`, `.ogv`, `.ogm` files;
- a sound: `.mp3`;
- a gist: `gist.github.com/…`.

**Polls.** `((poll Question` and a line per option (2 to 32), then
`))`. A line `multichoice N` (`мультивыбор N`) lets a voter choose `N`
(half of the options without a number); `closes dd.mm.yyyy`
(`закрывается dd.mm.yyyy`) ends the poll at noon, Moscow time.

**Tiles.** `((tile Title`, then the tile's link, then optionally an
image's link or `/icon/name`, then a description, then `))`. Tiles one
after another make a row.

### Errors

A text always reads; these are reported, each at its line:

- a block without its closing line: `((cut` without `))`, `>>` without
  `<<`, a code fence or `{{` without its end;
- a `))` or `<<` that closes nothing.

## The preview

`html::render` gives a fragment: each block has `data-line`, the line
it starts at, for a preview to follow the editor. `html::STYLE` is its
stylesheet, with VS Code's theme colors where they're defined;
`html::page` puts both in a page, with FontAwesome 6.1.2 from cdnjs.
Words the page shows itself are English: «Show» on a spoiler without a
title, «Poll», «Multiple choice», «Closes»; the site shows them in
Russian.

Unlike the site:

- videos and gists show as links, not players: the preview loads no
  other site;
- polls show their options, not votes; a countdown shows its dates.

## Tokens

`Document::tokens` are what an editor highlights, each a kind of markup
(`TokenKind`). `semantic::token_type` gives each a standard semantic
token type (bold `type`, accent 1 `event`, heading `class` …), and the
editor maps those to markup scopes for txt2, so a theme shows bold as
bold:

| markup | token type |
|---|---|
| markers (`**`, `\|`, `>`) | `operator` |
| block tags (`((cut`, `))`, `>>`, fences) | `keyword` |
| a block's title | `namespace` |
| heading | `class` |
| bold, italic, struck | `type`, `parameter`, `enumMember` |
| accents 1, 2, 3 | `event`, `method`, `macro` |
| offtop | `comment` |
| label | `decorator` |
| code | `string` |
| link text, link | `function`, `regexp` |
| user, project, search | `property` |
| icon | `enum` |
| quote | `interface` |
| list marker | `number` |
| progress bar | `typeParameter` |

## Layout

- `ast.rs`: the document: blocks, inline nodes, tokens;
- `block.rs`: blocks, line by line;
- `inline.rs`: the text inside a line;
- `html.rs`, `style.css`: the preview;
- `semantic.rs`: tokens for an editor;
- `main.rs`: the CLI.

## Tests

```sh
cargo test
```

`README.txt2` must parse without errors and render; every token stays
inside its line.
