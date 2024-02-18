# Aresty

A compiling template library for Rust.

Links:

- Crate: https://crates.io/crates/aresty
- Documentation: https://docs.rs/aresty

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
aresty = "0.1"
```

## Examples

### Sample .rst template

```html
<ol>
{{#for i in ints}}
    <li class="{{#if i % 6 == 0}}fizzbuzz{{#else if i % 2 == 0}}fizz{{#else if i % 3 == 0}}buzz{{#else}}none{{/if}}">{{i}}</li>
{{/for}}
</ol>
<ul>
{{#for o in opts}}
    <li>
        {{#match o}}
        {{=None}}Nothing at all
        {{=Some(s)}}It is a "{{s}}"
        {{/match}}
    </li>
{{/for}}
</ul>
```

### Ad-hoc macro usage

```rust
use std::io::Write;
use aresty::{aresty_render, escape::Escape, escape::NoEscape, Result};

fn hello_world(out: &mut impl Write) -> Result {
    let world = "World!";
    let result: Result = aresty_render!(out, NoEscape, "aresty_examples/src/hello_world.rst");
    result
}
```

### Proc macro on view struct

```rust
use aresty::{aresty, Result, Template};

#[aresty("aresty_examples/src/view.rst")]
struct View<'a> {
    ints: Vec<i32>,
    opts: &'a Vec<Option<String>>,
}

fn main() -> Result {
    let mut out = std::io::stdout();
    let view = View {
        ints: vec![1, 2, 3, 4, 5, 6, 60],
        opts: &vec![None, Some("thing & co".to_string())],
    };
    view.render_html(&mut out)?;
    Ok(())
}
```

## Supported tags

| Tag | effect |
| --- | ------ |
| <code>{{*expr*}}</code> | *`expr`* is evaluated, and printed escaped |
| <code>{{{*expr*}}}</code> | *`expr`* is evaluated, and printed as is (without escaping) |
| <code>{{!*expr*}}</code> | *`expr`* is evaluated, but not printed (i.e. `let`) |
| <code>{{#*block*}}</code> | *`block`* opened (i.e. `if`, `else`, `else if`, `for`, `match`, etc.) |
| <code>{{/*block*}}</code> | *`block`* closed |
| <code>{{=*expr*}}</code> | *`expr`* branch for the parent `match` block; does not need to be closed |