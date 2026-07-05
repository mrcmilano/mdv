# Heading One

Some **bold text**, some *italic text*, some ~~struck text~~, and one nested
combination: **bold *italic* text**.

## Heading Two

A paragraph with `inline code`, a link with a distinct url:
[the text](https://example.com/page), an autolink: <https://example.org>,
and an image: ![a small icon](icon.png).

### Heading Three

A horizontal rule follows.

---

#### Heading Four

Text before a hard break.  
Text after the hard break, and this line
continues via a soft break onto what was a separate source line.

##### Heading Five

CJK text: 你好世界. Emoji: 🎉🚀.

###### Heading Six

One 200-character unbroken word:
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa

## Code Blocks

```rust
fn main() {
    println!("hello");
}
```

```
plain fenced code with no language
```

    an indented code block

## Blockquotes

> A quoted paragraph with **bold** text inside it.
>
> > A nested quote inside the first one.

## Lists

- Top-level item one
  1. A nested ordered item
     - A deeply nested unordered item
- Top-level item two

Task list:

- [ ] An unchecked task
- [x] A checked task

## Raw HTML

<div>
A raw HTML block, rendered dim and verbatim.
</div>

Inline HTML like <span>this</span> stays dim inline.

## Footnotes

A sentence with a footnote reference[^note].

[^note]: The footnote's body text.
