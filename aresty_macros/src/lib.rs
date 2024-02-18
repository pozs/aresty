use proc_macro::TokenStream;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Ident, ItemStruct, LitStr, Result, Token};
use quote::quote;

#[proc_macro_attribute]
pub fn aresty(attr: TokenStream, input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let item = parse_macro_input!(input as ItemStruct);
    let ident = &item.ident;
    let lifetimes = item.generics.lifetimes().map(|_| quote!('_));
    let fields = item.fields.iter()
        .filter_map(|f| f.ident.as_ref()
            .map(|i| if let syn::Type::Reference(_) = f.ty {
                quote!(let #i = self.#i)
            } else {
                quote!(let #i = &self.#i)
            }));

    let tokens = quote! {
        #item

        impl ::aresty::Template for #ident<#( #lifetimes, )*> {
            fn render(&self, aresty_output: &mut impl ::std::io::Write, escape: &impl ::aresty::escape::Escape) -> ::aresty::Result {
                #( #fields; )*
                let result: ::aresty::Result = ::aresty_macros::aresty_render!(aresty_output, escape, #path);
                result
            }
        }
    };

    tokens.into()
}

struct RenderArgs {
    output: Ident,
    escape: Ident,
    path: LitStr,
}

impl Parse for RenderArgs {
    fn parse(args: ParseStream) -> Result<Self> {
        let output = args.parse()?;
        let _: Token![,] = args.parse()?;
        let escape = args.parse()?;
        let _: Token![,] = args.parse()?;
        let path = args.parse()?;
        Ok(RenderArgs {
            output,
            escape,
            path,
        })
    }
}

#[proc_macro]
pub fn aresty_render(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as RenderArgs);
    let template = RenderTemplate {
        contents: &read(args.path.value()),
        args: &args,
        block: None,
        have_match_arms: false,
    };
    let mut code = String::new();
    code.push('{');
    for statement in template {
        code.push_str(&statement);
    }
    code.push_str("::core::result::Result::Ok(())");
    code.push('}');
    code.parse().expect("Could not parse")
}

const BOM: &str = "\u{feff}";

fn read(path: String) -> String {
    match std::fs::read_to_string(&path) {
        Ok(mut contents) => {
            if contents.starts_with(BOM) {
                contents = contents[BOM.len()..].to_string()
            }
            contents
        },
        Err(error) => panic!("{path} cannot be read: {error:?}"),
    }
}

struct RenderTemplate<'a> {
    contents: &'a str,
    args: &'a RenderArgs,
    block: Option<&'a str>,
    have_match_arms: bool,
}

const START: &str = "{{";
const END: &str = "}}";
const BLOCK: u8 = b'#';
const RAW: u8 = b'{';
const EVAL: u8 = b'!';
const MATCH_BRANCH: u8 = b'=';
const END_BLOCK: u8 = b'/';
const RAW_END: &str = "}}}";

impl<'a> RenderTemplate<'a> {
    fn render_str(&mut self, until: usize) -> Option<String> {
        let str = &self.contents[..until];
        let output = &self.args.output;
        self.contents = &self.contents[until..];
        if !self.have_match_arms && Some("match") == self.block {
            if str.chars().all(char::is_whitespace) {
                return Some("".to_string());
            }
            panic!("Output before first match arm: {}", str);
        }
        Some(format!("write!({output}, \"{{}}\", {str:?})?;"))
    }

    fn render_code(&mut self) -> Option<String> {
        let mut new_contents = &self.contents[2..];
        let Some(next) = new_contents.bytes().position(|b| b != b' ' && (b'\x09' > b || b > b'\x0d')) else {
            panic!("Opened braces not closed");
        };
        new_contents = &new_contents[next..];
        match new_contents.bytes().nth(0) {
            None => panic!("Opened braces not closed"),
            Some(RAW) => self.render_raw(&new_contents[1..]),
            Some(EVAL) => self.render_eval(&new_contents[1..]),
            Some(BLOCK) => self.render_block(&new_contents[1..]),
            Some(MATCH_BRANCH) => self.render_match_branch(&new_contents[1..]),
            Some(END_BLOCK) => self.render_block_close(&new_contents[1..]),
            Some(_) => self.render_value(new_contents),
        }
    }

    fn render_raw(&mut self, new_contents: &'a str) -> Option<String> {
        self.contents = &new_contents;
        let idx = self.contents.find(RAW_END).expect("Opened raw braces not closed");
        let code = &self.contents[..idx];
        self.contents = &self.contents[idx + RAW_END.len()..];
        let output = &self.args.output;
        Some(format!("write!({output}, \"{{}}\", {code})?;"))
    }

    fn render_value(&mut self, new_contents: &'a str) -> Option<String> {
        self.contents = new_contents;
        let idx = self.contents.find(END).expect("Opened braces not closed");
        let code = &self.contents[..idx];
        self.contents = &self.contents[idx + END.len()..];
        let output = &self.args.output;
        let escape = &self.args.escape;
        Some(format!("{escape}.write({output}, &format!(\"{{}}\", {code}))?;"))
    }

    fn render_eval(&mut self, new_contents: &'a str) -> Option<String> {
        self.contents = &new_contents;
        let idx = self.contents.find(END).expect("Opened braces not closed");
        let code = &self.contents[..idx];
        self.contents = &self.contents[idx + END.len()..];
        self.skip_eol();
        let mut code = code.to_string();
        code.push(';');
        Some(code)
    }

    fn render_block(&mut self, new_contents: &'a str) -> Option<String> {
        let save = self.contents;
        self.contents = &new_contents;
        let idx = self.contents.find(END).expect("Opened braces not closed");
        let code = &self.contents[..idx];
        self.contents = &self.contents[idx + END.len()..];
        self.skip_eol();
        let mut block = Self::next_word(code);
        if block == "else" {
            if self.block.filter(|&b| b == "if").is_some() {
                self.contents = save;
                return None;
            }
            block = "if";
        }

        let template = &mut RenderTemplate {        
            contents: self.contents,
            args: self.args,
            block: Some(block),
            have_match_arms: false,
        };

        let mut code = if block == "block" {
            String::new()
        } else {
            code.to_string()
        };
        code.push('{');
        for statement in template.into_iter() {
            code.push_str(&statement);
        }
        code.push('}');
        if template.have_match_arms {
            code.push('}');
        }
        self.contents = template.contents;
        Some(code)
    }

    fn render_block_close(&mut self, new_contents: &'a str) -> Option<String> {
        self.contents = &new_contents;
        let idx = self.contents.find(END).expect("Opened braces not closed");
        let code = &self.contents[..idx];
        self.contents = &self.contents[idx + END.len()..];
        self.skip_eol();
        let input = Self::next_word(code);
        let Some(word) = self.block else {
            panic!("Trying to close {} in root context", input);
        };
        if input != word {
            panic!("Cannot close {}, most outer block is {}", input, word);
        }
        None
    }

    fn render_match_branch(&mut self, new_contents: &'a str) -> Option<String> {
        if Some("match") != self.block {
            panic!("Match branches can only be placed inside match blocks");
        }
        self.contents = &new_contents;
        let idx = self.contents.find(END).expect("Opened braces not closed");
        let code = &self.contents[..idx];
        self.contents = &self.contents[idx + END.len()..];
        self.skip_eol();

        let mut result = String::new();
        if self.have_match_arms {
            result.push('}');
        } else {
            self.have_match_arms = true;
        }
        result.push_str(code);
        result.push_str(" => {");
        Some(result)
    }

    fn skip_eol(&mut self) {
        self.skip_char(b'\r');
        self.skip_char(b'\n');
    }

    fn skip_char(&mut self, ch: u8) {
        if Some(ch) == self.contents.bytes().nth(0) {
            self.contents = &self.contents[1..];
        }
    }

    fn next_word(code: &str) -> &str {
        if let Some(start) = code.find(|c: char| !c.is_whitespace()) {
            let mut code = &code[start..];
            if let Some(end) = code.find(|c: char| !c.is_alphanumeric()) {
                code = &code[..end];
            }
            code
        } else {
            panic!("Invalid block")
        }
    }
}

impl Iterator for RenderTemplate<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.contents.is_empty() {
            if self.block.is_some() {
                panic!("The following blocks are not closed: {:?}", self.block.unwrap());
            }
            None
        } else {
            match self.contents.find(START) {
                None => self.render_str(self.contents.len()),
                Some(0) => self.render_code(),
                Some(idx) => self.render_str(idx),
            }
        }
    }
}
