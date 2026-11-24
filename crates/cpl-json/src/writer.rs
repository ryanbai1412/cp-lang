//! JSON written in the order it is given.

use crate::{write_string, Value};
use std::fmt::Write;

/// Writes JSON text in the order it is given: an object's keys stay where
/// they were written, unlike a [`Value`]'s, which are sorted. Strings are
/// escaped as [`Value`] escapes them, and a value is written as
/// [`crate::to_string`] writes it. Every object and array must be ended
/// before the writer is dropped.
pub struct Writer<'a> {
    out: &'a mut String,
    open: Vec<Open>,
}

#[derive(Clone, Copy)]
enum Open {
    Object { empty: bool, keyed: bool },
    Array { empty: bool },
}

/// A number the writer takes.
pub trait Number: std::fmt::Display {}
macro_rules! numbers {
    ($($t:ty),*) => { $(impl Number for $t {})* };
}
numbers!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize);

impl<'a> Writer<'a> {
    pub fn new(out: &'a mut String) -> Self {
        Writer {
            out,
            open: Vec::new(),
        }
    }

    fn item(&mut self) {
        match self.open.last_mut() {
            Some(Open::Array { empty }) => {
                if !*empty {
                    self.out.push(',');
                }
                *empty = false;
            }
            Some(Open::Object { keyed, .. }) => {
                assert!(
                    *keyed,
                    "cpl_json::Writer: a value in an object needs a key first"
                );
                *keyed = false;
            }
            None => {}
        }
    }

    /// The key of the next value in the open object.
    pub fn key(&mut self, key: &str) -> &mut Self {
        let open = self.open.last_mut();
        let ready = matches!(open, Some(Open::Object { keyed: false, .. }));
        assert!(
            ready,
            "cpl_json::Writer: key `{key}` outside an object or after another key"
        );
        if let Some(Open::Object { empty, keyed }) = open {
            if !*empty {
                self.out.push(',');
            }
            *empty = false;
            *keyed = true;
        }
        let _ = write_string(self.out, key);
        self.out.push(':');
        self
    }

    /// Opens an object, closed by [`Writer::end`].
    pub fn object(&mut self) -> &mut Self {
        self.item();
        self.out.push('{');
        self.open.push(Open::Object {
            empty: true,
            keyed: false,
        });
        self
    }

    /// Opens an array, closed by [`Writer::end`].
    pub fn array(&mut self) -> &mut Self {
        self.item();
        self.out.push('[');
        self.open.push(Open::Array { empty: true });
        self
    }

    /// Closes the innermost open object or array.
    pub fn end(&mut self) -> &mut Self {
        let open = self.open.pop();
        assert!(open.is_some(), "cpl_json::Writer: end with nothing open");
        let keyed = matches!(open, Some(Open::Object { keyed: true, .. }));
        assert!(
            !keyed,
            "cpl_json::Writer: an object ended after a key with no value"
        );
        self.out.push(if matches!(open, Some(Open::Array { .. })) {
            ']'
        } else {
            '}'
        });
        self
    }

    pub fn str(&mut self, text: &str) -> &mut Self {
        self.item();
        let _ = write_string(self.out, text);
        self
    }

    pub fn int(&mut self, n: impl Number) -> &mut Self {
        self.item();
        let _ = write!(self.out, "{n}");
        self
    }

    /// `x` with `places` digits after the point, or `null` when it is not finite.
    pub fn fixed(&mut self, x: f64, places: usize) -> &mut Self {
        self.item();
        if x.is_finite() {
            let _ = write!(self.out, "{x:.places$}");
        } else {
            self.out.push_str("null");
        }
        self
    }

    pub fn bool(&mut self, b: bool) -> &mut Self {
        self.item();
        self.out.push_str(if b { "true" } else { "false" });
        self
    }

    pub fn null(&mut self) -> &mut Self {
        self.item();
        self.out.push_str("null");
        self
    }

    /// `text` as a string, or `null` without one.
    pub fn opt_str(&mut self, text: Option<&str>) -> &mut Self {
        match text {
            Some(text) => self.str(text),
            None => self.null(),
        }
    }

    pub fn value(&mut self, value: &Value) -> &mut Self {
        self.item();
        let _ = write!(self.out, "{value}");
        self
    }

    /// A whole JSON text a writer wrote earlier, kept as text: a trace
    /// keeps each value's JSON once and writes it into every step.
    pub fn json(&mut self, text: &str) -> &mut Self {
        self.item();
        self.out.push_str(text);
        self
    }

    /// [`Writer::json`], or `null` without one.
    pub fn opt_json(&mut self, text: Option<&str>) -> &mut Self {
        match text {
            Some(text) => self.json(text),
            None => self.null(),
        }
    }
}

impl Drop for Writer<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(
                self.open.is_empty(),
                "cpl_json::Writer dropped with {} object(s) or array(s) still open",
                self.open.len()
            );
        }
    }
}

/// The JSON text `write` writes.
pub fn write_with(write: impl FnOnce(&mut Writer)) -> String {
    let mut out = String::new();
    write(&mut Writer::new(&mut out));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{obj, parse};

    #[test]
    fn keys_stay_in_the_order_written_and_parse_back_to_the_same_value() {
        let text = write_with(|w| {
            w.object().key("z").int(1usize).key("a").array();
            w.str("x<y\u{2028}\"q\"")
                .bool(false)
                .null()
                .value(&obj([("k", 2i64.into())]));
            w.end().key("m").opt_str(None).key("n").json("[1,2]").end();
        });
        assert_eq!(
            text,
            r#"{"z":1,"a":["x\u003cy\u2028\"q\"",false,null,{"k":2}],"m":null,"n":[1,2]}"#
        );
        let expected = obj([
            (
                "a",
                vec![
                    "x<y\u{2028}\"q\"".into(),
                    false.into(),
                    Value::Null,
                    obj([("k", 2i64.into())]),
                ]
                .into(),
            ),
            ("m", Value::Null),
            ("n", vec![1i64.into(), 2i64.into()].into()),
            ("z", 1i64.into()),
        ]);
        assert_eq!(parse(&text).unwrap(), expected);
        assert_eq!(crate::to_string(&expected).len(), text.len());
    }

    #[test]
    fn fixed_rounds_to_its_places_and_writes_null_when_not_finite() {
        let text = write_with(|w| {
            w.array().fixed(1.23456, 3).fixed(2.0, 1).fixed(f64::NAN, 2);
            w.fixed(f64::INFINITY, 2).end();
        });
        assert_eq!(text, "[1.235,2.0,null,null]");
    }

    #[test]
    #[should_panic(expected = "still open")]
    fn an_unclosed_object_is_caught() {
        write_with(|w| {
            w.object().key("a").array();
        });
    }

    #[test]
    #[should_panic(expected = "with no value")]
    fn a_key_with_no_value_is_caught() {
        write_with(|w| {
            w.object().key("a").end();
        });
    }

    #[test]
    #[should_panic(expected = "needs a key")]
    fn a_value_with_no_key_in_an_object_is_caught() {
        write_with(|w| {
            w.object().int(1).end();
        });
    }
}
