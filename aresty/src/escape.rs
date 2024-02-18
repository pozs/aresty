use super::*;
use std::any::type_name;

pub trait Escape {
    fn write(&self, to: &mut impl Write, val: &str) -> Result;
}

pub struct NoEscape;

impl Escape for NoEscape {
    fn write(&self, to: &mut impl Write, val: &str) -> Result {
        write!(to, "{}", val)
    }
}

pub struct Html;

impl Escape for Html {
    fn write(&self, to: &mut impl Write, val: &str) -> Result {
        let mut rem = val;
        loop {
            let escape = rem.find(|c| c == '"' || c == '&' || c == '<' || c == '>' || c == '@');
            if let Some(idx) = escape {
                write!(to, "{}", &rem[..idx])?;
                write!(to, "{}", match rem.as_bytes().get(idx).unwrap_or(&0) {
                    b'"' => "&quot;",
                    b'&' => "&amp;",
                    b'<' => "&lt;",
                    b'>' => "&gt;",
                    b'@' => "&#64;",
                    b => panic!("{} Unexpected character: {}", type_name::<Self>(), b),
                })?;
                rem = &rem[idx + 1..];
            } else {
                break;
            }
        }
        write!(to, "{}", rem)?;
        Ok(())
    }
}

pub struct Csv;

impl Escape for Csv {
    fn write(&self, to: &mut impl Write, val: &str) -> Result {
        let mut rem = val;
        let mut quote = rem.find(|c| c == '"');
        let other = rem.find(|c| c == '\n' || c == '\r' || c == ',' || c == ';');
        if quote != None || other != None {
            write!(to, "{}", '"')?;
            while let Some(idx) = quote {
                write!(to, "{}", &rem[..idx + 1])?;
                write!(to, "{}", '"')?;
                rem = &rem[idx + 1..];
                quote = rem.find(|c| c == '"');
            }
            write!(to, "{}{}", rem, '"')?;
        } else {
            write!(to, "{}", rem)?;
        }
        Ok(())
    }
}

pub struct Tsv;

impl Escape for Tsv {
    fn write(&self, to: &mut impl Write, val: &str) -> Result {
        let mut rem = val;
        loop {
            let escape = rem.find(|c| c == '\n' || c == '\r' || c == '\t' || c == '\\');
            if let Some(idx) = escape {
                write!(to, "{}", &rem[..idx])?;
                write!(to, "{}", match rem.as_bytes().get(idx).unwrap_or(&0) {
                    b'\n' => "\\n",
                    b'\r' => "\\r",
                    b'\t' => "\\t",
                    b'\\' => "\\\\",
                    b => panic!("{} Unexpected character: {}", type_name::<Self>(), b),
                })?;
                rem = &rem[idx + 1..];
            } else {
                break;
            }
        }
        write!(to, "{}", rem)?;
        Ok(())
    }
}

pub struct Url;

impl Escape for Url {
    fn write(&self, to: &mut impl Write, val: &str) -> Result {
        let mut rem = val;
        loop {
            let escape = rem.find(|c| {
                c == '!' || c == '#' || c == '$' || c == '&' || c == '\'' ||
                c == '(' || c == ')' || c == '*' || c == '+' || c == ',' ||
                c == '/' || c == ':' || c == ';' || c == '=' || c == '?' ||
                c == '@' || c == '[' || c == ']' || c == '%' || c == ' '
            });
            if let Some(idx) = escape {
                write!(to, "{}", &rem[..idx])?;
                write!(to, "{}", match rem.as_bytes().get(idx).unwrap_or(&0) {
                    b'!' => "%21",
                    b'#' => "%23",
                    b'$' => "%24",
                    b'&' => "%26",
                    b'\'' => "%27",
                    b'(' => "%28",
                    b')' => "%29",
                    b'*' => "%2A",
                    b'+' => "%2B",
                    b',' => "%2C",
                    b'/' => "%2F",
                    b':' => "%3A",
                    b';' => "%3B",
                    b'=' => "%3D",
                    b'?' => "%3F",
                    b'@' => "%40",
                    b'[' => "%5B",
                    b']' => "%5D",
                    b'%' => "%25",
                    b' ' => "+",
                    b => panic!("{} Unexpected character: {}", type_name::<Self>(), b),
                })?;
                rem = &rem[idx + 1..];
            } else {
                break;
            }
        }
        write!(to, "{}", rem)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        let mut buf = Vec::new();
        Html.write(&mut buf, "<i>Sample & \"test@example.com\"</a>").expect("write");
        assert_eq!(String::from_utf8(buf).expect("parse"), "&lt;i&gt;Sample &amp; &quot;test&#64;example.com&quot;&lt;/a&gt;");
    }

    #[test]
    fn test_csv_escape() {
        let mut buf = Vec::new();
        Csv.write(&mut buf, "val1,\"val2\"").expect("write");
        assert_eq!(String::from_utf8(buf).expect("parse"), "\"val1,\"\"val2\"\"\"");
    }

    #[test]
    fn test_tsv_escape() {
        let mut buf = Vec::new();
        Tsv.write(&mut buf, "\\val1\t\nval2").expect("write");
        assert_eq!(String::from_utf8(buf).expect("parse"), "\\\\val1\\t\\nval2");
    }

    #[test]
    fn test_url_escape() {
        let mut buf = Vec::new();
        Url.write(&mut buf, "a=b&c='d e'").expect("write");
        assert_eq!(String::from_utf8(buf).expect("parse"), "a%3Db%26c%3D%27d+e%27");
    }
}
