//! A run as one small entry per step and per change, with the whole state
//! every [`SNAPSHOT_EVERY`] steps, both taken while it runs. Step `k` is the
//! nearest snapshot at or before it plus the changes since; nothing is run
//! again.

use super::Step;
use std::sync::Arc;

pub(super) const SNAPSHOT_EVERY: usize = 1 << 10;

/// Text kept in [`Log::text`]: where its length starts, as a LEB128
/// prefix, so a change holds it in four bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Text(u32);

/// Bytes of [`Log::text`] with no length before them: one item of an
/// array kept as items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Piece {
    at: u32,
    len: u32,
}

/// A variable's value: its JSON, or for an array whose cells change one at
/// a time, the JSON of each cell, shared with the snapshots until one changes.
#[derive(Clone, Debug)]
enum Slot {
    Whole(Text),
    Items(Arc<Vec<Piece>>),
    /// An `Int`, kept raw and written as JSON when a step is shown.
    Int(i64),
    /// An array of `Int`s whose cells change one at a time, kept raw.
    Ints(Arc<Vec<i64>>),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Delta {
    /// A variable has this value now.
    Set(Var, Text),
    /// An item of an array variable has this value now: the text starts
    /// with the item's index, as LEB128.
    Item(Var, Text),
    /// An `Int` variable holds this value now.
    SmallInt(Var, i32),
    /// An `Int` variable holds [`Log::ints`] at this index now.
    Int(Var, u32),
    /// An item of an array of `Int`s holds a new value: [`Log::ints`] at
    /// this index holds the item's index in its high half and the value
    /// in its low half.
    SmallIntItem(Var, u32),
    /// As [`Delta::SmallIntItem`], with the index and the value in two.
    IntItem(Var, u32),
    /// A variable went out of scope.
    Unset(Var),
    /// The tree form of a `[V]?V` variable.
    Tree(Var, Text),
    Locals(Text),
    /// The locals are `Int`s: see [`Locals::Ints`].
    IntLocals(u32),
    /// The `Int` local at this position among [`Delta::IntLocals`]'s holds this value now.
    IntLocal(u16, i32),
    /// The steps from here are in this stage of [`Log::stages`].
    Stage(u32),
    /// The next line of [`Log::out`] was printed.
    Print,
    /// A test case starts: the output starts anew.
    Case,
}

const _: () = assert!(std::mem::size_of::<Delta>() == 8);

type Var = u16;

fn narrow(var: usize) -> Var {
    Var::try_from(var).expect("a trace names fewer than 65536 variables")
}

impl Delta {
    fn var(self) -> Option<u32> {
        match self {
            Delta::Set(var, _)
            | Delta::Item(var, _)
            | Delta::SmallInt(var, _)
            | Delta::Int(var, _)
            | Delta::SmallIntItem(var, _)
            | Delta::IntItem(var, _)
            | Delta::Unset(var)
            | Delta::Tree(var, _) => Some(var as u32),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Entry {
    /// The line, with [`ITEM_FOCUS`] set when the step's focus is the
    /// array item its last change wrote.
    line: u32,
    /// The changes of this step end here in [`Log::changes`].
    changes: u32,
}

const NONE: u32 = u32::MAX;
const ITEM_FOCUS: u32 = 1 << 31;

impl Entry {
    fn line(&self) -> u32 {
        self.line & !ITEM_FOCUS
    }
}

/// One running call: interned, so each step names its stack in four bytes.
#[derive(Clone, Debug)]
struct Frame {
    parent: u32,
    name: u32,
    /// Where its caller called it.
    line: u32,
    depth: u32,
}

/// What a step says beyond its line.
#[derive(Clone, Debug, Default)]
pub(super) struct Note {
    pub label: Option<String>,
    pub checkpoint: Option<String>,
    pub focus: Option<(usize, Vec<usize>)>,
}

/// The locals of a step: their JSON, or when each is an `Int`, kept raw:
/// the [`Text`] of their names, as JSON strings each after a newline, with
/// their values in [`State::local_ints`]. A [`Delta::IntLocals`] at `at`
/// holds the names' text in the high half of [`Log::ints`]`[at]` and how
/// many there are in its low half, then their values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Locals {
    Text(Text),
    Ints(Text),
}

impl Default for Locals {
    fn default() -> Locals {
        Locals::Text(Text::default())
    }
}

/// The state between two changes.
#[derive(Clone, Debug, Default)]
pub(super) struct State {
    values: Vec<Option<Slot>>,
    trees: Vec<Option<Text>>,
    locals: Locals,
    local_ints: Vec<i64>,
    stage: u32,
    /// The lines of [`Log::out`] this test case printed.
    out: (u32, u32),
}

#[derive(Clone, Debug)]
struct Snapshot {
    /// The first step it is before.
    step: u32,
    /// [`Log::changes`] up to here are in it.
    change: u32,
    state: State,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Log {
    source: String,
    entries: Vec<Entry>,
    changes: Vec<Delta>,
    text: Vec<u8>,
    /// The raw values of [`Delta::Int`], [`Delta::IntItem`] and [`Locals::Ints`].
    ints: Vec<i64>,
    /// The names of the last [`Locals::Ints`], and where they are kept.
    int_names: Option<(String, Text)>,
    snapshots: Vec<Snapshot>,
    /// What the snapshots hold, in bytes.
    snapshot_bytes: usize,
    frames: Vec<Frame>,
    /// Each step's call stack in [`Log::frames`], or `NONE`, as runs: the
    /// step a run starts at and its frame.
    frame_runs: Vec<(u32, u32)>,
    names: Vec<String>,
    stages: Vec<String>,
    /// By step, in order: most steps have none.
    notes: Vec<(u32, Note)>,
    out: Vec<Text>,
    /// The state after the last change: what the tracer compares with.
    now: State,
    /// Variables drawn as trees: their [`Delta::Tree`] forms are their values.
    pub drawn_as_tree: Vec<bool>,
    /// The stack of the last step: each call's name, as a pointer and
    /// interned, the line it was called from, and its frame.
    stack: Vec<(usize, u32, u32, u32)>,
    /// Steps at the top level name `main` as their only frame.
    pub record_frames: bool,
}

impl Log {
    pub fn new(source: &str, record_frames: bool) -> Log {
        let mut log = Log { source: source.to_string(), record_frames, ..Log::default() };
        log.locals_changed("{}");
        log
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn bytes(&self) -> usize {
        self.text.len()
            + self.changes.len() * std::mem::size_of::<Delta>()
            + self.ints.len() * std::mem::size_of::<i64>()
            + self.entries.len() * std::mem::size_of::<Entry>()
            + self.out.len() * std::mem::size_of::<Text>()
            + self.snapshot_bytes
    }

    fn keep(&mut self, text: &str) -> Text {
        let at = self.text.len() as u32;
        leb128(&mut self.text, text.len() as u32);
        self.text.extend_from_slice(text.as_bytes());
        Text(at)
    }

    fn keep_item(&mut self, index: u32, json: &str) -> Text {
        let at = self.text.len() as u32;
        leb128(&mut self.text, index);
        self.keep(json);
        Text(at)
    }

    fn piece_of(&self, text: Text) -> Piece {
        piece_of(&self.text, text)
    }

    fn raw(&self, piece: Piece) -> &[u8] {
        &self.text[piece.at as usize..(piece.at + piece.len) as usize]
    }

    fn get(&self, text: Text) -> &str {
        std::str::from_utf8(self.raw(self.piece_of(text))).unwrap_or_default()
    }

    /// Whether `slot` is `json`.
    fn holds(&self, slot: &Slot, json: &str) -> bool {
        let json = json.as_bytes();
        match slot {
            Slot::Whole(text) => self.raw(self.piece_of(*text)) == json,
            Slot::Items(items) => {
                let mut at = 0;
                let mut next = |bytes: &[u8]| {
                    let same = json.get(at..at + bytes.len()) == Some(bytes);
                    at += bytes.len();
                    same
                };
                next(b"[")
                    && items.iter().enumerate().all(|(i, item)| (i == 0 || next(b",")) && next(self.raw(*item)))
                    && next(b"]")
                    && at == json.len()
            }
            Slot::Int(_) | Slot::Ints(_) => slot_json(&self.text, slot) == std::str::from_utf8(json).unwrap_or_default(),
        }
    }

    fn push(&mut self, delta: Delta) {
        apply(&mut self.now, delta, &self.text, &self.ints);
        self.changes.push(delta);
    }

    /// `var` holds `json` now; a change only if it held something else.
    pub fn set(&mut self, var: usize, json: &str) {
        if !self.now.values.get(var).and_then(Option::as_ref).is_some_and(|slot| self.holds(slot, json)) {
            let text = self.keep(json);
            self.push(Delta::Set(narrow(var), text));
        }
    }

    /// `var` holds the `Int` `value` now.
    pub fn set_int(&mut self, var: usize, value: i64) {
        if !matches!(self.now.values.get(var), Some(Some(Slot::Int(now))) if *now == value) {
            let delta = match i32::try_from(value) {
                Ok(small) => Delta::SmallInt(narrow(var), small),
                Err(_) => {
                    self.ints.push(value);
                    Delta::Int(narrow(var), self.ints.len() as u32 - 1)
                }
            };
            self.push(delta);
        }
    }

    /// Item `index` of the array of `Int`s `var` holds `value` now. False
    /// when `var` does not hold such an array with that item.
    pub fn set_int_item(&mut self, var: usize, index: usize, value: i64) -> bool {
        let (Ok(small), Ok(at)) = (Var::try_from(var), i64::try_from(index)) else { return false };
        let Some(Some(slot)) = self.now.values.get_mut(var) else { return false };
        if let Slot::Whole(text) = slot {
            match ints_of(&self.text, piece_of(&self.text, *text)) {
                Some(items) => *slot = Slot::Ints(Arc::new(items)),
                None => return false,
            }
        }
        let Slot::Ints(items) = slot else { return false };
        match items.get(index) {
            None => false,
            Some(&now) if now == value => true,
            Some(_) => {
                let pos = self.ints.len() as u32;
                let delta = match (u32::try_from(at), i32::try_from(value)) {
                    (Ok(index), Ok(v)) => {
                        self.ints.push(((index as i64) << 32) | (v as u32 as i64));
                        Delta::SmallIntItem(small, pos)
                    }
                    _ => {
                        self.ints.extend([at, value]);
                        Delta::IntItem(small, pos)
                    }
                };
                self.push(delta);
                true
            }
        }
    }

    /// Item `index` of array `var` holds `json` now. False when `var` does
    /// not hold an array with that item, so the whole value is wanted.
    pub fn set_item(&mut self, var: usize, index: usize, json: &str) -> bool {
        let (Ok(small), Ok(at)) = (Var::try_from(var), u32::try_from(index)) else { return false };
        let item = match self.now.values.get(var) {
            Some(Some(Slot::Items(items))) => items.get(index).copied(),
            Some(Some(Slot::Whole(text))) => match split(&self.text, self.piece_of(*text)) {
                Some(items) => {
                    let item = items.get(index).copied();
                    self.now.values[var] = Some(Slot::Items(Arc::new(items)));
                    item
                }
                None => None,
            },
            _ => None,
        };
        let Some(item) = item else { return false };
        if self.raw(item) != json.as_bytes() {
            let text = self.keep_item(at, json);
            self.push(Delta::Item(small, text));
        }
        true
    }

    /// What `var` holds now, as JSON.
    #[cfg(debug_assertions)]
    pub fn value_now(&self, var: usize) -> Option<String> {
        self.value_in(&self.now, var)
    }

    pub fn unset(&mut self, var: usize) {
        if self.now.values.get(var).is_some_and(Option::is_some) {
            self.push(Delta::Unset(narrow(var)));
        }
    }

    pub fn set_tree(&mut self, var: usize, json: &str) {
        let now = self.now.trees.get(var).copied().flatten().map(|t| self.get(t));
        if now != Some(json) {
            let text = self.keep(json);
            self.push(Delta::Tree(narrow(var), text));
        }
    }

    pub fn locals_changed(&mut self, json: &str) {
        let fresh = self.entries.is_empty() && self.changes.is_empty();
        let same = || match self.now.locals {
            Locals::Text(text) => self.raw(self.piece_of(text)) == json.as_bytes(),
            Locals::Ints(_) => self.locals_json(&self.now) == json,
        };
        if fresh || !same() {
            let text = self.keep(json);
            self.push(Delta::Locals(text));
        }
    }

    /// The locals are `Int`s: `names` as in [`Locals::Ints`], and their values.
    pub fn locals_ints(&mut self, names: &str, values: &[i64]) {
        let named = match &self.int_names {
            Some((kept, text)) if kept == names => *text,
            _ => {
                let text = self.keep(names);
                self.int_names = Some((names.to_string(), text));
                text
            }
        };
        if self.now.locals == Locals::Ints(named) && self.now.local_ints.len() == values.len() {
            let changed = || self.now.local_ints.iter().zip(values).enumerate().filter(|(_, (now, v))| now != v);
            if changed().all(|(k, (_, &v))| u16::try_from(k).is_ok() && i32::try_from(v).is_ok())
                && changed().count() <= 1 + values.len() / 2
            {
                for (k, &v) in values.iter().enumerate() {
                    if self.now.local_ints[k] != v {
                        self.push(Delta::IntLocal(k as u16, v as i32));
                    }
                }
                return;
            }
        }
        let at = self.ints.len() as u32;
        self.ints.push(((named.0 as i64) << 32) | values.len() as i64);
        self.ints.extend_from_slice(values);
        self.push(Delta::IntLocals(at));
    }

    /// The names and the values of [`Locals::Ints`] at `at`.
    fn int_locals(&self, at: u32) -> (Text, &[i64]) {
        let at = at as usize;
        let head = self.ints[at];
        (Text((head >> 32) as u32), &self.ints[at + 1..at + 1 + (head as u32) as usize])
    }

    fn locals_json(&self, state: &State) -> String {
        match state.locals {
            Locals::Text(text) => self.get(text).to_string(),
            Locals::Ints(names) => {
                use std::fmt::Write;
                let (names, values) = (self.get(names), &state.local_ints);
                let mut json = String::from("{");
                for (i, (name, value)) in names.split('\n').skip(1).zip(values).enumerate() {
                    if i > 0 {
                        json.push(',');
                    }
                    let _ = write!(json, "{name}:{value}");
                }
                json.push('}');
                json
            }
        }
    }

    pub fn print(&mut self, line: &str) {
        let text = self.keep(line);
        self.out.push(text);
        self.push(Delta::Print);
    }

    pub fn new_case(&mut self) {
        self.push(Delta::Case);
    }

    /// The frame of a step whose stack is `stack`: `(fn, line it was
    /// called from)`, outermost first.
    pub fn frame(&mut self, stack: &[(&str, u32)]) -> u32 {
        let kept = self
            .stack
            .iter()
            .zip(stack)
            .take_while(|(&(at, _, line, _), (name, called))| at == name.as_ptr() as usize && line == *called)
            .count();
        self.stack.truncate(kept);
        let mut parent = self.stack.last().map_or(NONE, |top| top.3);
        for (name, line) in &stack[kept..] {
            let id = match self.names.iter().position(|known| known == name) {
                Some(id) => id as u32,
                None => {
                    self.names.push(name.to_string());
                    self.names.len() as u32 - 1
                }
            };
            let depth = if parent == NONE { 1 } else { self.frames[parent as usize].depth + 1 };
            self.frames.push(Frame { parent, name: id, line: *line, depth });
            parent = self.frames.len() as u32 - 1;
            self.stack.push((name.as_ptr() as usize, id, *line, parent));
        }
        parent
    }

    pub fn stage(&mut self, stage: &str) -> u32 {
        match self.stages.iter().position(|known| known == stage) {
            Some(at) => at as u32,
            None => {
                self.stages.push(stage.to_string());
                self.stages.len() as u32 - 1
            }
        }
    }

    /// A step ends: its changes are the ones made since the last.
    pub fn step(
        &mut self,
        line: u32,
        frame: Option<u32>,
        stage: u32,
        label: Option<String>,
        checkpoint: Option<String>,
        focus: Option<(usize, &[usize])>,
    ) {
        let at = self.entries.len() as u32;
        let mut focus = focus;
        let mut line = line;
        if let Some((var, path)) = focus {
            let item = self.item_written(self.changes_start(at as usize)..self.changes.len());
            if path.len() == 1 && item == Some((var, path[0])) {
                focus = None;
                line |= ITEM_FOCUS;
            }
        }
        if label.is_some() || checkpoint.is_some() || focus.is_some() {
            let focus = focus.map(|(var, path)| (var, path.to_vec()));
            self.notes.push((at, Note { label, checkpoint, focus }));
        }
        if stage != self.now.stage || at == 0 {
            self.push(Delta::Stage(stage));
        }
        let frame = frame.unwrap_or(NONE);
        if self.frame_runs.last().is_none_or(|&(_, last)| last != frame) {
            self.frame_runs.push((at, frame));
        }
        self.entries.push(Entry { line, changes: self.changes.len() as u32 });
    }

    /// A snapshot before the next step, when one is due: every
    /// [`SNAPSHOT_EVERY`] steps, and no sooner than the changes since the
    /// last one outweigh it four times, so snapshots hold a fifth of the log at most.
    pub fn snapshot_if_due(&mut self) {
        let step = self.entries.len() as u32;
        let (since, changes) = self.snapshots.last().map_or((step, self.changes.len()), |s| {
            (step - s.step, self.changes.len() - s.change as usize)
        });
        if (since as usize) < SNAPSHOT_EVERY {
            return;
        }
        let items: usize = self.now.values.iter().flatten().map(|slot| match slot {
            Slot::Items(items) => items.len(),
            Slot::Ints(items) => items.len(),
            Slot::Whole(_) | Slot::Int(_) => 0,
        }).sum();
        let bytes = (self.now.values.len() + self.now.trees.len() + self.now.local_ints.len()) * 8
            + items * std::mem::size_of::<Piece>();
        if changes * std::mem::size_of::<Delta>() >= 4 * bytes {
            let state = self.now.clone();
            self.snapshot_bytes += bytes;
            self.snapshots.push(Snapshot { step, change: self.changes.len() as u32, state });
        }
    }

    /// The last step is taken back: its changes go to the next one.
    pub fn reopen(&mut self) {
        if self.entries.pop().is_some() {
            let at = self.entries.len() as u32;
            if self.notes.last().is_some_and(|&(step, _)| step == at) {
                self.notes.pop();
            }
            if self.frame_runs.last().is_some_and(|&(start, _)| start == at) {
                self.frame_runs.pop();
            }
        }
    }

    pub fn draw_as_tree(&mut self, var: usize) {
        if self.drawn_as_tree.len() <= var {
            self.drawn_as_tree.resize(var + 1, false);
        }
        self.drawn_as_tree[var] = true;
    }

    /// The array item the last [`Delta::Item`] of `changes` wrote.
    fn item_written(&self, changes: std::ops::Range<usize>) -> Option<(usize, usize)> {
        self.changes[changes].iter().rev().find_map(|delta| match *delta {
            Delta::Item(var, text) => Some((var as usize, item_of(&self.text, text).0 as usize)),
            Delta::SmallIntItem(var, _) | Delta::IntItem(var, _) => {
                Some((var as usize, int_item(*delta, &self.ints).0))
            }
            _ => None,
        })
    }

    /// Gives back what the buffers hold beyond their contents.
    pub fn shrink(&mut self) {
        self.entries.shrink_to_fit();
        self.changes.shrink_to_fit();
        self.text.shrink_to_fit();
        self.ints.shrink_to_fit();
        self.snapshots.shrink_to_fit();
        self.frame_runs.shrink_to_fit();
        self.notes.shrink_to_fit();
        self.out.shrink_to_fit();
    }

    fn note(&self, step: usize) -> Option<&Note> {
        let at = self.notes.binary_search_by_key(&(step as u32), |&(at, _)| at).ok()?;
        Some(&self.notes[at].1)
    }

    pub fn checkpoints(&self) -> impl Iterator<Item = (usize, &str)> {
        self.notes.iter().filter_map(|(at, note)| Some((*at as usize, note.checkpoint.as_deref()?)))
    }

    pub fn last(&self) -> Option<(u32, Option<&Note>, u32)> {
        let at = self.entries.len().checked_sub(1)?;
        let entry = &self.entries[at];
        Some((entry.line(), self.note(at), self.now.stage))
    }

    /// Keeps only the last `keep` steps.
    pub fn keep_last(&mut self, keep: usize) {
        let drop = self.entries.len().saturating_sub(keep);
        if drop == 0 {
            return;
        }
        let state = self.state_before(drop);
        let first = self.changes_start(drop);
        let mut log = Log {
            source: std::mem::take(&mut self.source),
            frames: std::mem::take(&mut self.frames),
            names: std::mem::take(&mut self.names),
            stages: std::mem::take(&mut self.stages),
            drawn_as_tree: std::mem::take(&mut self.drawn_as_tree),
            stack: std::mem::take(&mut self.stack),
            record_frames: self.record_frames,
            ..Log::default()
        };
        let moved = |text: Text, log: &mut Log| log.keep(self.get(text));
        let move_int_locals = |at: u32, log: &mut Log| {
            let (names, values) = self.int_locals(at);
            let names = log.keep(self.get(names));
            let to = log.ints.len() as u32;
            log.ints.push(((names.0 as i64) << 32) | values.len() as i64);
            log.ints.extend_from_slice(values);
            to
        };
        let locals = match state.locals {
            Locals::Text(text) => Locals::Text(moved(text, &mut log)),
            Locals::Ints(names) => Locals::Ints(moved(names, &mut log)),
        };
        let mut start =
            State { locals, local_ints: state.local_ints.clone(), stage: state.stage, ..State::default() };
        for slot in &state.values {
            let slot = slot.as_ref().map(|slot| match slot {
                Slot::Whole(text) => Slot::Whole(moved(*text, &mut log)),
                Slot::Items(items) => Slot::Items(Arc::new(
                    items
                        .iter()
                        .map(|item| {
                            let at = log.text.len() as u32;
                            log.text.extend_from_slice(self.raw(*item));
                            Piece { at, len: item.len }
                        })
                        .collect(),
                )),
                Slot::Int(v) => Slot::Int(*v),
                Slot::Ints(items) => Slot::Ints(items.clone()),
            });
            start.values.push(slot);
        }
        start.trees = state.trees.iter().map(|t| t.map(|t| moved(t, &mut log))).collect();
        for line in &self.out[state.out.0 as usize..] {
            let text = moved(*line, &mut log);
            log.out.push(text);
        }
        start.out = (0, state.out.1 - state.out.0);
        log.snapshots.push(Snapshot { step: 0, change: 0, state: start.clone() });
        log.now = start;
        for delta in &self.changes[first..] {
            let delta = match *delta {
                Delta::Set(var, text) => Delta::Set(var, moved(text, &mut log)),
                Delta::Item(var, text) => {
                    let (index, item) = item_of(&self.text, text);
                    let json = std::str::from_utf8(self.raw(item)).unwrap_or_default();
                    Delta::Item(var, log.keep_item(index, json))
                }
                Delta::Int(var, at) => {
                    log.ints.push(self.ints[at as usize]);
                    Delta::Int(var, log.ints.len() as u32 - 1)
                }
                Delta::SmallIntItem(var, at) => {
                    log.ints.push(self.ints[at as usize]);
                    Delta::SmallIntItem(var, log.ints.len() as u32 - 1)
                }
                Delta::IntItem(var, at) => {
                    let at = at as usize;
                    log.ints.extend_from_slice(&self.ints[at..at + 2]);
                    Delta::IntItem(var, log.ints.len() as u32 - 2)
                }
                Delta::Tree(var, text) => Delta::Tree(var, moved(text, &mut log)),
                Delta::Locals(text) => Delta::Locals(moved(text, &mut log)),
                Delta::IntLocals(at) => Delta::IntLocals(move_int_locals(at, &mut log)),
                other => other,
            };
            log.changes.push(delta);
            apply(&mut log.now, delta, &log.text, &log.ints);
        }
        let shift = first as u32;
        for entry in &self.entries[drop..] {
            log.entries.push(Entry { changes: entry.changes - shift, ..*entry });
        }
        log.notes = std::mem::take(&mut self.notes)
            .into_iter()
            .filter(|&(step, _)| step as usize >= drop)
            .map(|(step, note)| (step - drop as u32, note))
            .collect();
        let kept = self.frame_runs.partition_point(|&(start, _)| start as usize <= drop).max(1) - 1;
        log.frame_runs = self.frame_runs[kept..].iter()
            .map(|&(start, frame)| ((start as usize).saturating_sub(drop) as u32, frame))
            .collect();
        *self = log;
    }

    fn changes_start(&self, step: usize) -> usize {
        match step {
            0 => 0,
            _ => self.entries[step - 1].changes as usize,
        }
    }

    /// The state when step `step` starts.
    fn state_before(&self, step: usize) -> State {
        let to = self.changes_start(step);
        let at = self.snapshots.partition_point(|s| s.change as usize <= to);
        let (mut state, from) = match at.checked_sub(1).map(|i| &self.snapshots[i]) {
            Some(snapshot) => (snapshot.state.clone(), snapshot.change as usize),
            None => (State::default(), 0),
        };
        self.replay_into(&mut state, from, to);
        state
    }

    fn replay_into(&self, state: &mut State, from: usize, to: usize) {
        for delta in &self.changes[from..to] {
            apply(state, *delta, &self.text, &self.ints);
        }
    }

    /// The state when step `step` has run.
    pub fn state_at(&self, step: usize) -> State {
        let mut state = self.state_before(step);
        self.replay_into(&mut state, self.changes_start(step), self.entries[step].changes as usize);
        state
    }

    fn frame_at(&self, step: usize) -> Option<u32> {
        if step >= self.entries.len() {
            return None;
        }
        let run = self.frame_runs.partition_point(|&(start, _)| start as usize <= step);
        Some(self.frame_runs[run - 1].1)
    }

    pub fn line(&self, step: usize) -> Option<u32> {
        self.entries.get(step).map(Entry::line)
    }

    /// How many calls are running at `step`, the top level being one.
    pub fn depth(&self, step: usize) -> usize {
        match self.frame_at(step) {
            Some(frame) if frame != NONE => self.frames[frame as usize].depth as usize + 1,
            _ => 1,
        }
    }

    /// The variables step `step` changed, and whether it changed the
    /// locals or the output.
    pub fn touched(&self, step: usize) -> (Vec<usize>, bool) {
        let mut vars = Vec::new();
        let mut other = false;
        for delta in &self.changes[self.changes_start(step)..self.entries[step].changes as usize] {
            match delta.var() {
                Some(var) => vars.push(var as usize),
                None => other |= !matches!(delta, Delta::Stage(_)),
            }
        }
        (vars, other)
    }

    pub fn advance(&self, state: &mut State, step: usize) {
        self.replay_into(state, self.changes_start(step), self.entries[step].changes as usize);
    }

    pub fn start(&self, step: usize) -> State {
        self.state_before(step)
    }

    /// A variable's value in `state`, as JSON.
    pub fn value_in(&self, state: &State, var: usize) -> Option<String> {
        if var == STDOUT {
            let mut json = String::from("[");
            for (i, line) in self.out[state.out.0 as usize..state.out.1 as usize].iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                super::string(&mut json, self.get(*line));
            }
            json.push(']');
            return Some(json);
        }
        let tree = self.drawn_as_tree.get(var).copied().unwrap_or(false);
        if let Some(tree) = state.trees.get(var).copied().flatten().filter(|_| tree) {
            return Some(self.get(tree).to_string());
        }
        Some(slot_json(&self.text, state.values.get(var)?.as_ref()?))
    }

    pub fn locals_in(&self, state: &State) -> String {
        self.locals_json(state)
    }

    /// Step `step`, given the state when it has run.
    pub fn step_of(&self, step: usize, state: &State, vars: usize) -> Step {
        let entry = &self.entries[step];
        let note = self.note(step);
        let mut values = vec![(STDOUT, Arc::from(self.value_in(state, STDOUT).unwrap_or_default()))];
        for var in 1..vars {
            if let Some(json) = self.value_in(state, var) {
                values.push((var, Arc::from(json)));
            }
        }
        let mut frames = Vec::new();
        let mut at = self.frame_at(step).unwrap_or(NONE);
        if at != NONE {
            frames.push((self.names[self.frames[at as usize].name as usize].clone(), entry.line()));
            while at != NONE {
                let frame = &self.frames[at as usize];
                let caller = match frame.parent {
                    NONE => "main".to_string(),
                    parent => self.names[self.frames[parent as usize].name as usize].clone(),
                };
                frames.push((caller, frame.line));
                at = frame.parent;
            }
        } else if self.record_frames {
            frames.push(("main".to_string(), entry.line()));
        }
        let item_focus = || {
            let (var, index) = self.item_written(self.changes_start(step)..self.changes_start(step + 1))?;
            Some((var, vec![index]))
        };
        let focus = note
            .and_then(|n| n.focus.clone())
            .or_else(|| if entry.line & ITEM_FOCUS != 0 { item_focus() } else { None })
            .filter(|(var, _)| !self.drawn_as_tree.get(*var).copied().unwrap_or(false));
        Step {
            line: entry.line(),
            label: note.and_then(|n| n.label.clone()).unwrap_or_else(|| line_text(&self.source, entry.line())),
            stage: self.stages.get(state.stage as usize).cloned().unwrap_or_default(),
            values,
            locals: self.locals_in(state),
            checkpoint: note.and_then(|n| n.checkpoint.clone()),
            focus,
            frames,
        }
    }
}

/// The variable that is the output.
pub(super) const STDOUT: usize = 0;

pub(super) fn line_text(source: &str, line: u32) -> String {
    let text = source.lines().nth(line as usize - 1).unwrap_or("").trim();
    let mut short: String = text.chars().take(80).collect();
    if short.len() < text.len() {
        short.push('…');
    }
    short
}

fn leb128(bytes: &mut Vec<u8>, mut n: u32) {
    while n >= 0x80 {
        bytes.push(n as u8 | 0x80);
        n >>= 7;
    }
    bytes.push(n as u8);
}

/// The LEB128 number at `at`, and where it ends.
fn read_leb128(bytes: &[u8], mut at: usize) -> (u32, usize) {
    let (mut n, mut shift) = (0u32, 0);
    loop {
        let byte = bytes[at];
        at += 1;
        n |= u32::from(byte & 0x7f) << shift;
        shift += 7;
        if byte < 0x80 {
            return (n, at);
        }
    }
}

fn piece_of(bytes: &[u8], text: Text) -> Piece {
    let (len, at) = read_leb128(bytes, text.0 as usize);
    Piece { at: at as u32, len }
}

/// The index and the JSON of a [`Delta::Item`].
fn item_of(bytes: &[u8], text: Text) -> (u32, Piece) {
    let (index, at) = read_leb128(bytes, text.0 as usize);
    (index, piece_of(bytes, Text(at as u32)))
}

/// The items of the JSON array `whole`, or `None` when it is not one.
fn split(bytes: &[u8], whole: Piece) -> Option<Vec<Piece>> {
    let text = &bytes[whole.at as usize..(whole.at + whole.len) as usize];
    let inner = text.strip_prefix(b"[")?.strip_suffix(b"]")?;
    let mut items = Vec::new();
    if inner.is_empty() {
        return Some(items);
    }
    let (mut depth, mut quoted, mut escaped, mut start) = (0usize, false, false, 0usize);
    let base = whole.at as usize + 1;
    for (i, &byte) in inner.iter().enumerate() {
        match byte {
            _ if escaped => escaped = false,
            b'\\' if quoted => escaped = true,
            b'"' => quoted = !quoted,
            _ if quoted => {}
            b'[' | b'{' => depth += 1,
            b']' | b'}' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => {
                items.push(Piece { at: (base + start) as u32, len: (i - start) as u32 });
                start = i + 1;
            }
            _ => {}
        }
    }
    items.push(Piece { at: (base + start) as u32, len: (inner.len() - start) as u32 });
    Some(items)
}

/// A slot's value as JSON.
fn slot_json(bytes: &[u8], slot: &Slot) -> String {
    let raw = |piece: &Piece| std::str::from_utf8(&bytes[piece.at as usize..(piece.at + piece.len) as usize]).unwrap_or_default();
    match slot {
        Slot::Whole(text) => raw(&piece_of(bytes, *text)).to_string(),
        Slot::Int(v) => v.to_string(),
        Slot::Items(items) => {
            let mut json = String::with_capacity(2 + items.iter().map(|i| i.len as usize + 1).sum::<usize>());
            json.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                json.push_str(raw(item));
            }
            json.push(']');
            json
        }
        Slot::Ints(items) => {
            use std::fmt::Write;
            let mut json = String::with_capacity(2 + items.len() * 4);
            json.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                let _ = write!(json, "{item}");
            }
            json.push(']');
            json
        }
    }
}

/// The items of the JSON array `whole` when each is an integer that
/// reads back as it is written.
fn ints_of(bytes: &[u8], whole: Piece) -> Option<Vec<i64>> {
    split(bytes, whole)?
        .iter()
        .map(|item| {
            let text = std::str::from_utf8(&bytes[item.at as usize..(item.at + item.len) as usize]).ok()?;
            let v: i64 = text.parse().ok()?;
            (v.to_string() == text).then_some(v)
        })
        .collect()
}

/// The index and the value a [`Delta::SmallIntItem`] or [`Delta::IntItem`] writes.
fn int_item(delta: Delta, ints: &[i64]) -> (usize, i64) {
    match delta {
        Delta::SmallIntItem(_, at) => {
            let packed = ints[at as usize];
            ((packed >> 32) as usize, packed as i32 as i64)
        }
        Delta::IntItem(_, at) => (ints[at as usize] as usize, ints[at as usize + 1]),
        _ => unreachable!("not an item of Ints"),
    }
}

fn apply(state: &mut State, delta: Delta, bytes: &[u8], ints: &[i64]) {
    let grow = |slots: &mut Vec<Option<Text>>, var: Var| {
        if slots.len() <= var as usize {
            slots.resize(var as usize + 1, None);
        }
    };
    match delta {
        Delta::Set(var, text) => {
            if state.values.len() <= var as usize {
                state.values.resize(var as usize + 1, None);
            }
            state.values[var as usize] = Some(Slot::Whole(text));
        }
        Delta::Item(var, text) => {
            let (index, item) = item_of(bytes, text);
            let slot = &mut state.values[var as usize];
            if let Some(Slot::Whole(whole)) = slot {
                let items = split(bytes, piece_of(bytes, *whole)).expect("an array");
                *slot = Some(Slot::Items(Arc::new(items)));
            }
            if let Some(Slot::Items(items)) = slot {
                Arc::make_mut(items)[index as usize] = item;
            }
        }
        Delta::Int(var, at) => {
            if state.values.len() <= var as usize {
                state.values.resize(var as usize + 1, None);
            }
            state.values[var as usize] = Some(Slot::Int(ints[at as usize]));
        }
        Delta::SmallInt(var, value) => {
            if state.values.len() <= var as usize {
                state.values.resize(var as usize + 1, None);
            }
            state.values[var as usize] = Some(Slot::Int(value.into()));
        }
        Delta::SmallIntItem(var, _) | Delta::IntItem(var, _) => {
            let (index, value) = int_item(delta, ints);
            let slot = &mut state.values[var as usize];
            if let Some(Slot::Whole(whole)) = slot {
                *slot = Some(Slot::Ints(Arc::new(ints_of(bytes, piece_of(bytes, *whole)).expect("an array of Ints"))));
            }
            if let Some(Slot::Ints(items)) = slot {
                Arc::make_mut(items)[index] = value;
            }
        }
        Delta::Stage(stage) => state.stage = stage,
        Delta::Unset(var) => {
            if let Some(slot) = state.values.get_mut(var as usize) {
                *slot = None;
            }
            if let Some(slot) = state.trees.get_mut(var as usize) {
                *slot = None;
            }
        }
        Delta::Tree(var, text) => {
            grow(&mut state.trees, var);
            state.trees[var as usize] = Some(text);
        }
        Delta::Locals(text) => state.locals = Locals::Text(text),
        Delta::IntLocals(at) => {
            let (at, head) = (at as usize, ints[at as usize]);
            state.locals = Locals::Ints(Text((head >> 32) as u32));
            state.local_ints.clear();
            state.local_ints.extend_from_slice(&ints[at + 1..at + 1 + (head as u32) as usize]);
        }
        Delta::IntLocal(k, value) => state.local_ints[k as usize] = value.into(),
        // `out.1` is how many lines the run has printed.
        Delta::Print => state.out.1 += 1,
        Delta::Case => state.out.0 = state.out.1,
    }
}

/// Steps rebuilt in order, each from the one before.
pub struct Steps<'a> {
    log: &'a Log,
    state: State,
    next: usize,
    end: usize,
    vars: usize,
}

impl<'a> Steps<'a> {
    pub(super) fn new(log: &'a Log, range: std::ops::Range<usize>, vars: usize) -> Steps<'a> {
        let state = if range.is_empty() { State::default() } else { log.start(range.start) };
        Steps { log, state, next: range.start, end: range.end, vars }
    }
}

impl Iterator for Steps<'_> {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        if self.next >= self.end {
            return None;
        }
        self.log.advance(&mut self.state, self.next);
        self.next += 1;
        Some(self.log.step_of(self.next - 1, &self.state, self.vars))
    }
}
