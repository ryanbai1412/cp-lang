//! Every diagnostic message, as data. An emit site names a row and passes
//! its arguments, `msgs::lexer::TAB.error(span, &[])`, so a message's
//! wording lives here and nowhere else (`diag_text_outside_registry`).
//! Each file of rows belongs to the crate area that emits them.

use crate::{Code, Diagnostic, Span};
use std::fmt::{Display, Write};

pub mod check;
pub mod foreign;
pub mod lexer;
pub mod lint;
pub mod stmt;
pub mod syntax;

/// One message: its code and its text, with a `{}` hole per argument.
#[derive(Debug)]
pub struct Msg {
    #[cfg(test)]
    pub key: &'static str,
    pub code: Code,
    pub text: &'static str,
}

impl Msg {
    pub fn say(&self, args: &[&dyn Display]) -> String {
        fill(self.text, args)
    }

    pub fn error(&self, span: Span, args: &[&dyn Display]) -> Diagnostic {
        Diagnostic::error(self.code, span, self.say(args))
    }

    pub fn warning(&self, span: Span, args: &[&dyn Display]) -> Diagnostic {
        Diagnostic::warning(self.code, span, self.say(args))
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
    ($($(#[$doc:meta])* $name:ident = $code:tt, $text:expr;)*) => {
        $($(#[$doc])* pub static $name: $crate::msgs::Msg = $crate::msgs::Msg {
            #[cfg(test)]
            key: stringify!($name),
            code: $crate::msgs::msgs!(@code $code),
            text: $text,
        };)*
        pub static ALL: &[&$crate::msgs::Msg] = &[$(&$name),*];
    };
}
pub(crate) use msgs;

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> impl Iterator<Item = (&'static str, &'static Msg)> {
        [
            ("check", check::ALL),
            ("foreign", foreign::ALL),
            ("lexer", lexer::ALL),
            ("lint", lint::ALL),
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
            let n = holes(m.text);
            let args: Vec<String> = (0..n).map(|i| format!("<arg{i}>")).collect();
            let args: Vec<&dyn Display> = args.iter().map(|a| a as &dyn Display).collect();
            let said = m.say(&args);
            assert!(!said.trim().is_empty(), "{file}::{} is empty", m.key);
            for (i, _) in args.iter().enumerate() {
                assert!(
                    said.contains(&format!("<arg{i}>")),
                    "{file}::{}: {said}",
                    m.key
                );
            }
            let stray = said.replace("{{", "").replace("}}", "");
            assert!(
                !stray.contains("{}"),
                "{file}::{} leaves a hole: {said}",
                m.key
            );
            assert!(
                m.code.0.starts_with('L') || crate::registry::lookup(m.code.0).is_some(),
                "{file}::{} has no registry row for {}",
                m.key,
                m.code.0
            );
        }
    }

    #[test]
    fn no_message_is_written_twice() {
        let mut seen = std::collections::HashMap::new();
        for (file, m) in all() {
            if let Some(other) = seen.insert(m.text, (file, m.key)) {
                panic!("{file}::{} repeats {other:?}: {:?}", m.key, m.text);
            }
        }
    }

    #[test]
    fn fill_places_arguments_and_braces() {
        assert_eq!(fill("`{}` and {{x}} {}", &[&1, &"b"]), "`1` and {x} b");
    }
}
