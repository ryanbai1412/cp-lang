//! Syntax the 0.3 design has and this version does not implement yet.
//!
//! The parser reads each form, so the statement and the names it declares
//! survive, and reports it once as "not implemented yet: <feature> (see
//! <section>)" with what to write today. When a form is implemented, its
//! entry goes: the test that each `example` is reported fails until then.

pub struct Planned {
    pub feature: &'static str,
    /// The section of the design that specifies it.
    pub see: &'static str,
    /// What to write in this version instead.
    pub now: &'static str,
    /// A program whose one error is this one.
    pub example: &'static str,
}

pub const BOUNDED_STR: Planned = Planned {
    feature: "a string type with a bounded length, `Str(len: …)`",
    see: "design/spec/29-strings.md §29.1",
    now: "read a `Str` and check its `.len` where it is used",
    example: "let s: Str(len: 1..=10) = read()\nprint(s.len)\n",
};

pub const PLANNED: &[&Planned] = &[&BOUNDED_STR];
