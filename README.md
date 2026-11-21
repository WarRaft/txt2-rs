# txt2-rs

txt2, the markup of [XGM](https://xgm.guru): a parser, an HTML renderer
for a preview and semantic tokens for an editor. The
[wc3-rs](../wc3-rs) editor highlights `.txt2` files with it and shows
them beside the text, as Markdown's preview does.

The parser never fails: what isn't markup is text, and what's wrong (a
block without its closing line, a `))` closing nothing) is reported with
its line. Every block knows the line it starts at, so a preview can
follow the editor.

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

## Syntax

From the site's documentation (`test/`, copied from its pages; their
noise, like «» glued to an example, is the site's) and
from articles written in it. Where the two differ, articles win: they
use single markers (`?byte?`, `+0+`) the documentation only hints at.

### Text

| markup | is |
|---|---|
| `*word*`, `**several words**` | bold |
| `_word_`, `__several words__` | italic |
| `-word-`, `--several words--` | struck |
| `!word!` `!!…!!`, `+word+` `++…++`, `?word?` `??…??` | accents 1 (red), 2 (green), 3 (blue) |
| `// aside //`, `// to the end of the line` | offtop |
| `[[[label]]]`, `[!!…!!]`, `[++…++]`, `[??…??]` | labels |
| `{{text}}` | as it is: code, no formatting |
| `https://…` (and `ftp`, `mailto`, `tg`, `steam` …) | a link, shown without its scheme |
| `__text__ (url)`, `_word_ (url)`, `Word (url)`, `[text](url)` | a link with text |
| `@name` (`+` for spaces), `~name`, `~~two words~~`, `@p/project` | a user, a project |
| `w:`, `в:`, `g:`, `я:`, `s:` and a term | Wikipedia (en, ru), Google, Yandex, the site's search |
| `/иконка/name`, `/icon/name/3x` | a FontAwesome 6 icon (solid), size 1x to 5x: `<i class="fa fa-solid fa-name fa-3x">`; `html::page` links the font from cdnjs, an editor's preview brings its own |

A single marker wraps one word: it starts after something that isn't a
letter or digit and ends before one, with no space inside (`C++`,
`snake_case`, `xgm-team` stay text). A double one may hold spaces; the
first character after it isn't one.

### Blocks

Each starts a line; the ones that open run to their closing line.

| markup | is |
|---|---|
| `# Title`, `## …`; `== Title ==`, `===Title` | headings, the level the count |
| `---` (one or more `-`) | a rule |
| `> quote`, `> > nested` | a quote |
| `((цитата` … `))` | a quote of several lines |
| `>> title` … `<<`, `((кат title` … `))` | a spoiler; without a title, «раскрыть» |
| `((центр` … `))`, `((вправо` … `))` | centered, right-aligned |
| `((макет` … `))` with `((блок33`, `((блок50`, `((блок66` … `))` in it | a row of columns of that width |
| ```` ```lang ```` … ```` ``` ````, `""` … `""`, `((код lang` … `))` | code; a fence with a space after it is code too |
| `{{` … `}}` | text as it is, its lines kept |
| `- item`, `* item`, `1. item`, `1) item`, `2.1. item` | lists; lines indented more belong to the item above |
| rows with `\|` (or tabs), after a blank line | a table: `---` after the header rows; `<` joins a cell to its left, `^` to the one above; `>>> ` right, `<> ` centered; `{{\|}}` a `\|` in a cell |
| `[ 5% ]`, `[ 2 / 5 ]`, `[ 4 / 3 - 19 ]`, `[ 28.11.2023 / 27.11.2025 ]` and text | a progress bar |
| a link alone on its line | an image (`.png`, `.jpg`, `.webp`, `.avif` …; `<- `, `-> `, `<> ` before it: left, right, center; ` - caption` after it, or ` (url)` for an image that links there; two or more on lines one after another, none aligned, make a gallery), a video (YouTube, VK, Rutube, Twitch, Vimeo …, `.mp4`, `.webm`), a sound (`.mp3`), a gist |
| `((опрос title` … `))` | a poll: a line per option, `мультивыбор N`, `закрывается dd.mm.yyyy` |
| `((плитка title` … `))` | a tile: its link, then an image or `/иконка/…`, then a description |

English tags work as well: `cut` or `spoiler`, `quote`, `center`,
`right`, `layout`, `block33` …, `code`, `poll`, `tile`; `multichoice`,
`closes`. Of these the documentation names `layout`, `block33`, `closes`
and `/icon/`; the others are guesses.

## Not like the site

- Videos and gists show as links, not players: the preview loads no
  other site.
- Polls show their options, not votes; a date countdown shows its dates.

## Tokens

`semantic::token_type` gives each kind of markup a standard token type
(bold `type`, accent 1 `event`, heading `class` …): the editor maps them
to markup scopes for txt2, so a theme shows bold as bold.

## Tests

```sh
cargo test
```

The pages in `test/` must parse and render, and their tokens stay in
their lines.
