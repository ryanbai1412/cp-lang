//! The `view` object every editor-facing `--format json` report carries:
//! what an editor draws, already laid out, so each editor renders it as is
//! and only clips labels to its width (spec/tools/cli.md, "The view").

use cpl_json::Writer;

/// What one report shows in an editor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct View {
    /// The output panel's text, as the command's own text output lays it out.
    pub panel: String,
    pub status: Option<Status>,
    pub lines: Vec<Line>,
    pub diagnostic: Option<Diagnostic>,
    /// The cases a test tree lists, in the order they ran.
    pub tests: Vec<Test>,
}

/// The verdict badge, such as `2 of 3 samples passed`.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    /// 0-based.
    pub line: u32,
    pub ok: bool,
    pub label: String,
    pub detail: Option<String>,
}

/// The mark at the end of one source line.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// 0-based.
    pub line: u32,
    pub kind: Kind,
    /// The full label; the editor clips it to its width.
    pub label: String,
    /// Markdown.
    pub hover: Option<String>,
}

/// What a line's mark says, strongest first: an editor showing several
/// reports keeps the strongest mark on each line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Error,
    Differs,
    Agrees,
    Input,
    Live,
}

impl Kind {
    /// Every kind, strongest first.
    pub const ALL: [Kind; 5] = [
        Kind::Error,
        Kind::Differs,
        Kind::Agrees,
        Kind::Input,
        Kind::Live,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Error => "error",
            Kind::Differs => "differs",
            Kind::Agrees => "agrees",
            Kind::Input => "input",
            Kind::Live => "live",
        }
    }
}

/// One case as a test tree shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Test {
    pub name: String,
    /// `passed`, `failed` (a wrong answer) or `errored` (it crashed, ran out
    /// of time or memory, or the judge failed).
    pub outcome: &'static str,
    pub time_ms: u64,
    /// Why it did not pass, in one line.
    pub message: Option<String>,
    /// The answer a wrong one is compared with, when the case has one.
    pub expected: Option<String>,
    pub output: String,
}

/// The one problem the report puts in the editor's problem list.
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    /// 0-based.
    pub line: u32,
    /// 0-based.
    pub col: Option<u32>,
    pub code: Option<String>,
    pub message: String,
    pub help: Option<String>,
    /// Who reported it, such as `cpl (i1)`.
    pub source: Option<String>,
}

impl View {
    /// The view as a value, for a report built as a `cpl_json::Value`.
    pub fn to_value(&self) -> cpl_json::Value {
        let written = cpl_json::write_with(|w| {
            w.object();
            self.write(w);
            w.end();
        });
        cpl_json::parse(&written)
            .unwrap_or_default()
            .at(&["view"])
            .clone()
    }

    /// Writes `"view": {...}` into the object `w` has open.
    pub fn write(&self, w: &mut Writer) {
        w.key("view").object().key("panel").str(&self.panel);
        if let Some(status) = &self.status {
            w.key("status").object().key("line").int(status.line);
            w.key("ok").bool(status.ok).key("label").str(&status.label);
            if let Some(detail) = &status.detail {
                w.key("detail").str(detail);
            }
            w.end();
        }
        if !self.lines.is_empty() {
            w.key("lines").array();
            for line in &self.lines {
                w.object().key("line").int(line.line);
                w.key("kind").str(line.kind.as_str());
                w.key("label").str(&line.label);
                if let Some(hover) = &line.hover {
                    w.key("hover").str(hover);
                }
                w.end();
            }
            w.end();
        }
        if let Some(d) = &self.diagnostic {
            w.key("diagnostic").object().key("line").int(d.line);
            if let Some(col) = d.col {
                w.key("col").int(col);
            }
            if let Some(code) = &d.code {
                w.key("code").str(code);
            }
            w.key("message").str(&d.message);
            for (key, text) in [("help", &d.help), ("source", &d.source)] {
                if let Some(text) = text {
                    w.key(key).str(text);
                }
            }
            w.end();
        }
        if !self.tests.is_empty() {
            w.key("tests").array();
            for test in &self.tests {
                w.object().key("name").str(&test.name);
                w.key("outcome").str(test.outcome);
                w.key("time_ms").int(test.time_ms);
                for (key, text) in [("message", &test.message), ("expected", &test.expected)] {
                    if let Some(text) = text {
                        w.key(key).str(text);
                    }
                }
                w.key("output").str(&test.output).end();
            }
            w.end();
        }
        w.end();
    }
}

/// `text` in a fenced Markdown code block of `lang`.
pub fn code_block(text: &str, lang: &str) -> String {
    let fence = if text.contains("```") { "~~~~" } else { "```" };
    format!("{fence}{lang}\n{text}\n{fence}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(view: &View) -> String {
        cpl_json::write_with(|w| {
            w.object();
            view.write(w);
            w.end();
        })
    }

    #[test]
    fn writes_only_the_fields_it_has() {
        assert_eq!(written(&View::default()), r#"{"view":{"panel":""}}"#);
        let view = View {
            panel: "6\n".into(),
            status: Some(Status {
                line: 0,
                ok: false,
                label: "1 of 2".into(),
                detail: None,
            }),
            lines: vec![Line {
                line: 3,
                kind: Kind::Live,
                label: "i1: 6".into(),
                hover: Some("h".into()),
            }],
            diagnostic: Some(Diagnostic {
                line: 4,
                col: Some(2),
                code: Some("R0301".into()),
                message: "m".into(),
                help: None,
                source: Some("cpl (i1)".into()),
            }),
            tests: vec![Test {
                name: "2".into(),
                outcome: "failed",
                time_ms: 3,
                message: Some("WA".into()),
                expected: None,
                output: "6\n".into(),
            }],
        };
        assert_eq!(
            written(&view),
            concat!(
                r#"{"view":{"panel":"6\n","status":{"line":0,"ok":false,"label":"1 of 2"},"#,
                r#""lines":[{"line":3,"kind":"live","label":"i1: 6","hover":"h"}],"#,
                r#""diagnostic":{"line":4,"col":2,"code":"R0301","message":"m","source":"cpl (i1)"},"#,
                r#""tests":[{"name":"2","outcome":"failed","time_ms":3,"message":"WA","output":"6\n"}]}}"#
            )
        );
    }

    #[test]
    fn kinds_order_strongest_first() {
        assert!(Kind::Error < Kind::Differs && Kind::Input < Kind::Live);
        assert_eq!(code_block("a ``` b", "cp"), "~~~~cp\na ``` b\n~~~~");
    }
}
