pub mod escape;

pub use aresty_macros::{aresty, aresty_render};
use std::io::{Write, Error};
pub type Result = std::result::Result<(), Error>;

pub trait Template {
    fn render_text(&self, to: &mut impl Write) -> Result {
        self.render(to, &escape::NoEscape)
    }

    fn render_html(&self, to: &mut impl Write) -> Result {
        self.render(to, &escape::Html)
    }

    fn render(&self, to: &mut impl Write, escape: &impl escape::Escape) -> Result;
}
