use std::io::Write;
use aresty::{aresty, aresty_render, escape::Escape, escape::NoEscape, Result, Template};

fn hello_world(out: &mut impl Write) -> Result {
    let world = "World!";
    let result: Result = aresty_render!(out, NoEscape, "aresty_examples/src/hello_world.rst");
    result
}

#[aresty("aresty_examples/src/view.rst")]
struct View<'a> {
    ints: Vec<i32>,
    opts: &'a Vec<Option<String>>,
}

fn main() -> Result {
    let mut out = std::io::stdout();
    hello_world(&mut out)?;
    println!();
    let view = View {
        ints: vec![1, 2, 3, 4, 5, 6, 60],
        opts: &vec![None, Some("thing & co".to_string())],
    };
    view.render_html(&mut out)?;
    Ok(())
}
