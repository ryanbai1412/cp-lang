//! Every diagnostic message, as data. An emit site names a row and passes
//! its arguments, `msgs::lexer::TAB.error(span, &[])`, so a message's
//! wording lives here and nowhere else (`diag_text_outside_registry`).
//! Each file of rows belongs to the crate area that emits them.

use crate::{Code, Diagnostic, Span};
use std::fmt::{Display, Write};

pub mod check;
pub mod foreign;
pub mod interp;
pub mod ir;
pub mod lexer;
pub mod lint;
pub mod lint_rules;
pub mod stmt;
pub mod syntax;

/// Bytes of a code (`E0302`, `L0566`) in a table's packed code column.
pub const CODE_LEN: usize = 5;

/// One file's rows: its codes packed `CODE_LEN` bytes a row, and its texts
/// back to back in one `str`, each ending at `ends[i]`. A row is an index, so
/// the wasm build holds no per-row statics.
#[doc(hidden)]
#[derive(Debug)]
pub struct Rows {
    #[cfg(test)]
    pub keys: &'static [&'static str],
    pub codes: &'static [u8],
    pub texts: &'static str,
    pub ends: &'static [u16],
}

/// One message: its code and its text, with a `{}` hole per argument.
#[derive(Debug, Clone, Copy)]
pub struct Msg {
    rows: &'static Rows,
    i: u16,
}

#[doc(hidden)]
pub const fn pack_codes<const M: usize>(codes: &[&str]) -> [u8; M] {
    let mut out = [0; M];
    let mut r = 0;
    while r < codes.len() {
        let c = codes[r].as_bytes();
        assert!(c.len() == CODE_LEN, "a code is CODE_LEN bytes");
        let mut k = 0;
        while k < CODE_LEN {
            out[r * CODE_LEN + k] = c[k];
            k += 1;
        }
        r += 1;
    }
    out
}

/// Where `name` is in `names`, worked out at compile time.
#[doc(hidden)]
pub const fn index_of(names: &[&str], name: &str) -> u16 {
    let mut i = 0;
    loop {
        assert!(i < names.len(), "a row's name is in its file");
        let (a, b) = (names[i].as_bytes(), name.as_bytes());
        let mut k = 0;
        while k < a.len() && a.len() == b.len() && a[k] == b[k] {
            k += 1;
        }
        if k == a.len() && a.len() == b.len() {
            return i as u16;
        }
        i += 1;
    }
}

#[doc(hidden)]
pub const fn ends<const N: usize>(lens: &[usize]) -> [u16; N] {
    let mut out = [0; N];
    let mut at = 0;
    let mut r = 0;
    while r < N {
        at += lens[r];
        assert!(at <= u16::MAX as usize, "a file's texts fit u16 offsets");
        out[r] = at as u16;
        r += 1;
    }
    out
}

impl Msg {
    #[doc(hidden)]
    pub const fn new(rows: &'static Rows, i: u16) -> Msg {
        Msg { rows, i }
    }

    pub fn code(&self) -> Code {
        let at = self.i as usize * CODE_LEN;
        Code(std::str::from_utf8(&self.rows.codes[at..at + CODE_LEN]).unwrap_or_default())
    }

    pub fn text(&self) -> &'static str {
        let i = self.i as usize;
        let start = if i == 0 {
            0
        } else {
            self.rows.ends[i - 1] as usize
        };
        &self.rows.texts[start..self.rows.ends[i] as usize]
    }

    #[cfg(test)]
    pub fn key(&self) -> &'static str {
        self.rows.keys[self.i as usize]
    }

    pub fn say(&self, args: &[&dyn Display]) -> String {
        fill(self.text(), args)
    }

    pub fn error(&self, span: Span, args: &[&dyn Display]) -> Diagnostic {
        Diagnostic::error(self.code(), span, self.say(args))
    }

    pub fn warning(&self, span: Span, args: &[&dyn Display]) -> Diagnostic {
        Diagnostic::warning(self.code(), span, self.say(args))
    }

    /// The row as a warning under `code`: for a lint a library declares
    /// with `@lint`, whose code is its own name.
    pub fn warning_as(&self, code: Code, span: Span, args: &[&dyn Display]) -> Diagnostic {
        Diagnostic::warning(code, span, self.say(args))
    }

    /// The row said with `args`, for a crate's own funnel to report.
    pub fn at(&self, args: &[&dyn Display]) -> Said {
        Said {
            code: self.code(),
            text: self.say(args),
        }
    }
}

/// A row's text with its arguments filled. Only a row makes one, so a funnel
/// that takes a `Said` takes no text of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    code: Code,
    text: String,
}

impl Said {
    pub fn code(&self) -> Code {
        self.code
    }

    pub fn text(self) -> String {
        self.text
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The same text under the code a site picks from the types, e.g. the
    /// wrong domain for a position where another domain's is wanted.
    pub fn with_code(self, code: Code) -> Said {
        Said { code, ..self }
    }

    /// `row` said with `args` and then this text, keeping this code: a row
    /// that places another message, such as where in a library it arose.
    pub fn wrapped(self, row: Msg, args: &[&dyn Display]) -> Said {
        let mut all = args.to_vec();
        all.push(&self.text);
        let text = row.say(&all);
        Said {
            code: self.code,
            text,
        }
    }

    pub fn error(self, span: Span) -> Diagnostic {
        Diagnostic::error(self.code, span, self.text)
    }

    pub fn warning(self, span: Span) -> Diagnostic {
        Diagnostic::warning(self.code, span, self.text)
    }
}

/// `text` with each `{}` replaced by the next argument; `{{` and `}}` are
/// a brace.
pub(crate) fn fill(text: &str, args: &[&dyn Display]) -> String {
    let mut out = String::new();
    let mut args = args.iter();
    let mut rest = text;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let step = if tail.starts_with("{}") {
            if let Some(arg) = args.next() {
                let _ = write!(out, "{arg}");
            }
            2
        } else {
            out.push_str(&tail[..1]);
            if tail.starts_with("{{") || tail.starts_with("}}") {
                2
            } else {
                1
            }
        };
        rest = &tail[step..];
    }
    out.push_str(rest);
    out
}

/// `NAME = CODE, "text with {} holes";` rows, and `ALL` listing them. A
/// lint's row names its code as a string, `NAME = "L0566", ".."`: lint
/// codes are cpl-lint's, which checks every row's code is one of its lints.
macro_rules! msgs {
    (@code $code:ident) => { $crate::codes::$code };
    (@code $code:literal) => { $crate::Code($code) };
    ($($(#[$doc:meta])* $name:ident = $code:tt, $text:literal;)*) => {
        const NAMES: &[&str] = &[$(stringify!($name)),*];
        const N: usize = NAMES.len();
        static ROWS: $crate::msgs::Rows = $crate::msgs::Rows {
            #[cfg(test)]
            keys: &[$(stringify!($name)),*],
            codes: &$crate::msgs::pack_codes::<{ N * $crate::msgs::CODE_LEN }>(
                &[$($crate::msgs::msgs!(@code $code).0),*],
            ),
            texts: concat!($($text),*),
            ends: &$crate::msgs::ends::<N>(&[$($text.len()),*]),
        };
        $($(#[$doc])* pub const $name: $crate::msgs::Msg = $crate::msgs::Msg::new(&ROWS, $crate::msgs::index_of(NAMES, stringify!($name)));)*
        pub const ALL: &[$crate::msgs::Msg] = &[$($name),*];
    };
}
pub(crate) use msgs;

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> impl Iterator<Item = (&'static str, Msg)> {
        [
            ("check", check::ALL),
            ("foreign", foreign::ALL),
            ("interp", interp::ALL),
            ("ir", ir::ALL),
            ("lexer", lexer::ALL),
            ("lint", lint::ALL),
            ("lint_rules", lint_rules::ALL),
            ("stmt", stmt::ALL),
            ("syntax", syntax::ALL),
        ]
        .into_iter()
        .flat_map(|(file, rows)| rows.iter().map(move |m| (file, *m)))
    }

    fn holes(text: &str) -> usize {
        text.replace("{{", "")
            .replace("}}", "")
            .matches("{}")
            .count()
    }

    #[test]
    fn every_message_renders() {
        for (file, m) in all() {
            let n = holes(m.text());
            let args: Vec<String> = (0..n).map(|i| format!("<arg{i}>")).collect();
            let args: Vec<&dyn Display> = args.iter().map(|a| a as &dyn Display).collect();
            let said = m.say(&args);
            assert!(!said.trim().is_empty(), "{file}::{} is empty", m.key());
            for (i, _) in args.iter().enumerate() {
                assert!(
                    said.contains(&format!("<arg{i}>")),
                    "{file}::{}: {said}",
                    m.key()
                );
            }
            let stray = said.replace("{{", "").replace("}}", "");
            assert!(
                !stray.contains("{}"),
                "{file}::{} leaves a hole: {said}",
                m.key()
            );
            assert!(
                m.code().0.starts_with('L') || crate::registry::lookup(m.code().0).is_some(),
                "{file}::{} has no registry row for {}",
                m.key(),
                m.code().0
            );
        }
    }

    #[test]
    fn no_message_is_written_twice() {
        let mut seen = std::collections::HashMap::new();
        for (file, m) in all() {
            if let Some(other) = seen.insert((m.code().0, m.text()), (file, m.key())) {
                panic!("{file}::{} repeats {other:?}: {:?}", m.key(), m.text());
            }
        }
    }

    #[test]
    fn fill_places_arguments_and_braces() {
        assert_eq!(fill("`{}` and {{x}} {}", &[&1, &"b"]), "`1` and {x} b");
    }
}
