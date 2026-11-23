//! What a run leaves in its top-level variables: the editors show it
//! beside the lines declaring them.

use super::{cells, render, Live, Mode, Recording, View};
use crate::value::Value;
use cpl_diag::{LineIndex, Span};
use cpl_ir::Ty;

/// Each top-level variable's value when the run ended, at the line
/// declaring it: what [`Recording::lives`] holds. `visible` says which
/// the run declared.
pub(super) fn last_values(
    view: &View,
    visible: impl Fn(usize) -> bool,
    line: impl Fn(Span) -> u32,
) -> Vec<Live> {
    let mut lives = Vec::new();
    for id in 0..view.globals.len() {
        let global = &view.program.globals[id];
        let value = &view.globals[id];
        if global.span.end == 0 || !visible(id) || matches!(value, Value::Unit | Value::Moved) {
            continue;
        }
        if matches!(global.ty, Ty::Unit | Ty::Fn(..) | Ty::DomainVal | Ty::Error) {
            continue;
        }
        let mut text = String::new();
        if cells(value) > 200 {
            text = format!("{} values", cells(value));
        } else {
            render(&mut text, value, &global.ty, view, Mode::Text);
        }
        lives.push(Live {
            line: line(global.span),
            name: global.name.clone(),
            value: text,
        });
    }
    lives.sort_by_key(|live| live.line);
    lives
}

/// The recording of an untraced run of the file `source`: only its
/// [`last_values`]. A global holds no value until it is declared.
pub(crate) fn last_values_of(view: &View, source: &str) -> Recording {
    let lines = LineIndex::new(source);
    Recording {
        lives: last_values(view, |_| true, |span| lines.line_of(span.start) + 1),
        ..Recording::default()
    }
}
