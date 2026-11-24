//! Lowering of a checked program to a single C++17 file.

mod bounds;
mod built;
mod capfn;
mod checks;
mod compact;
mod devirt;
mod divrem;
mod emit;
mod expr;
mod flat;
mod fold;
mod fuse;
mod globals;
mod inline;
mod lend;
mod names;
mod niche;
mod optfn;
mod order;
mod presize;
mod prune;
mod ranges;
mod rowcache;
mod runtime;
mod scope;
mod show;
mod sink;
mod site;
mod splice;
mod split;
mod stmt;
mod unordered;
mod unpack;
mod uses;
mod walk;
mod width;

pub use checks::Checks;
pub mod explain;
pub use devirt::{explain_dropped_fields, ClosureLowering, ClosureMode};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// The C++17 file for `program`. `name` is the source file's name and
/// `source` its text, which opens the file as a comment so that whoever
/// reads the C++ can read what it was written from. With no `source` the
/// file opens with a note saying it was left out.
pub fn emit(program: &cpl_ir::Program, name: &str, source: Option<&str>) -> String {
    emit_for(program, name, source, &Target::default())
}

/// What the C++ is written for: the judge it is submitted to.
#[derive(Clone, Debug)]
pub struct Target {
    /// A line for the header that names the judge.
    pub stamp: Option<String>,
    /// The stack the program runs with on a thread of its own; `None` on
    /// a judge that allows no threads, where it has the stack of `main`.
    pub thread_stack: Option<u64>,
    /// Whether the judge runs Windows, whose threads are not POSIX ones.
    pub windows: bool,
    /// The runtime checks the build keeps.
    pub checks: Checks,
}

impl Default for Target {
    fn default() -> Target {
        Target {
            stamp: None,
            thread_stack: Some(1 << 30),
            windows: false,
            checks: Checks::default(),
        }
    }
}

/// Where the bytes of a file go, in the order the file has them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Part {
    /// The opening comment: the source, or the note that it is left out.
    Header,
    Includes,
    /// The runtime pieces the program leans on, and how it prints values.
    Runtime,
    Types,
    Globals,
    /// The fns of imported modules, the stdlib's among them.
    Library,
    /// The file's own fns, closures and top-level code.
    Yours,
    /// Namespaces, declarations ahead and `main`.
    Glue,
}

impl Part {
    pub fn describe(self) -> &'static str {
        match self {
            Part::Header => "header",
            Part::Includes => "includes",
            Part::Runtime => "runtime",
            Part::Types => "types",
            Part::Globals => "globals",
            Part::Library => "library fns",
            Part::Yours => "your code",
            Part::Glue => "glue",
        }
    }
}

/// One named piece of a file and its length there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Size {
    pub part: Part,
    pub name: String,
    pub bytes: usize,
}

/// Where the bytes of a file of [`emit_within_for`] go: every piece of it,
/// with what is not in any piece as [`Part::Glue`], so that the lengths add
/// up to its length. `source` is the header's, if it kept it.
pub fn sizes(
    program: &cpl_ir::Program,
    name: &str,
    source: Option<&str>,
    target: &Target,
    fitted: &Fitted,
) -> Vec<Size> {
    let total = fitted.cpp.len();
    let program = prepared(program, target);
    let emitter = emit::Emitter::new(&program, target);
    let (_, chunks) = emitter.file_with_chunks(name, source);
    let fit = |part: Part, text: &str| {
        let text = match fitted.comments || part == Part::Header {
            true => text.to_string(),
            false => uncommented(text, false),
        };
        match (fitted.compact, fitted.narrow) {
            (true, _) => compact::compact(&text, part == Part::Header, &fitted.names).len(),
            (false, true) => narrow(&narrow(&text)).len(),
            (false, false) => narrow(&text).len(),
        }
    };
    let mut sizes: Vec<Size> = chunks
        .iter()
        .filter(|(_, _, text)| !text.is_empty())
        .map(|(part, name, text)| Size {
            part: *part,
            name: unmangled(name),
            bytes: match fitted.folded.get(&names::fn_name(name)) {
                Some(Some(template)) => fit(*part, template),
                Some(None) => 0,
                None => fit(*part, text),
            },
        })
        .filter(|size| size.bytes > 0)
        .collect();
    let named: usize = sizes.iter().map(|size| size.bytes).sum();
    sizes.push(Size {
        part: Part::Glue,
        name: "namespaces and main".into(),
        bytes: total.saturating_sub(named),
    });
    sizes
}

/// A fn's name as its source wrote it: `Rooted.path`, not the name of one
/// instance, `Rooted_D0_V_Int_7.path_Arr1_84`. A part of an instance's name
/// starts with `_` and a capital or a digit.
fn unmangled(name: &str) -> String {
    let cut = |segment: &str| {
        let bytes = segment.as_bytes();
        let at = (1..bytes.len())
            .find(|&at| {
                bytes[at - 1] == b'_' && bytes[at].is_ascii_uppercase() | bytes[at].is_ascii_digit()
            })
            .map_or(segment.len(), |at| at - 1);
        segment[..at].to_string()
    };
    name.split('.').map(cut).collect::<Vec<_>>().join(".")
}

/// `program` with what the emitter decides once for the whole file.
fn prepared(program: &cpl_ir::Program, target: &Target) -> cpl_ir::Program {
    let mut program = program.clone();
    (program.unordered_maps, program.unordered_sets) = unordered::find(&program);
    program.wide_domains = width::wide_domains(&program);
    prune::prune(&mut program, target.checks != Checks::None);
    splice::splice(&mut program);
    unpack::unpack(&mut program);
    optfn::known_options(&mut program);
    split::split(&mut program);
    capfn::known_captures(&mut program);
    // A closure `capfn` freed of its capture can now split its struct.
    split::split(&mut program);
    // A param `optfn` knows is `None` can leave a field only ever `None`.
    prune::prune(&mut program, target.checks != Checks::None);
    program
}

/// What a judge build emits unless `--checks` says: no traps (decisions.md 299).
fn judge_target() -> Target {
    Target {
        checks: Checks::None,
        ..Target::default()
    }
}

/// How each fn-typed site of the judge build's C++ for `program` is
/// called, read off the program the emitter writes (after pruning).
/// A closure the file never calls is dropped, whatever holds it, and a
/// struct the file does not define has no sites.
pub fn explain_closures(program: &cpl_ir::Program) -> Vec<ClosureLowering> {
    let target = judge_target();
    let program = prepared(program, &target);
    let (called, written) = emit::Emitter::new(&program, &target).called_and_written();
    let mut lowerings = devirt::explain_closures(&program, &written);
    for lowering in &mut lowerings {
        if let ClosureMode::StaticDirect { closure } = lowering.mode {
            if !called.contains(&closure) {
                lowering.mode = ClosureMode::Unread;
            }
        }
    }
    lowerings
}

/// What `cpl explain` reads off the judge build's C++, written once:
/// [`explain_closures`], [`explain_copies`] and the dropped fields.
pub struct Lowered {
    pub closures: Vec<ClosureLowering>,
    pub copies: Vec<(cpl_diag::Span, String, &'static str)>,
    pub dropped: Vec<String>,
}

pub fn explain_lowered(program: &cpl_ir::Program) -> Lowered {
    let target = Target::default();
    let prepared = prepared(program, &target);
    let (cpp, copies, called, written) = emit::Emitter::new(&prepared, &target).lowered();
    let mut closures = devirt::explain_closures(&prepared, &written);
    for lowering in &mut closures {
        if let ClosureMode::StaticDirect { closure } = lowering.mode {
            if !called.contains(&closure) {
                lowering.mode = ClosureMode::Unread;
            }
        }
    }
    Lowered {
        closures,
        copies,
        dropped: devirt::dropped_fields(program, &cpp),
    }
}

/// Each heap value the judge build's C++ for `program` copies: the span
/// of the source that is copied, what it is and why the copy must own it.
pub fn explain_copies(program: &cpl_ir::Program) -> Vec<(cpl_diag::Span, String, &'static str)> {
    let target = judge_target();
    let program = prepared(program, &target);
    emit::Emitter::new(&program, &target).copies()
}

/// [`emit`] for a judge.
pub fn emit_for(
    program: &cpl_ir::Program,
    name: &str,
    source: Option<&str>,
    target: &Target,
) -> String {
    let program = prepared(program, target);
    let emitter = emit::Emitter::new(&program, target);
    narrow(&emitter.file(name, source))
}

/// [`emit_for`] with every statement counting its starts and the time until
/// the next one starts, the books `cpl_interp::profile` keeps. The tally is
/// written when the program exits, to the file named by `CPL_PROFILE`, a
/// line `site hits nanos` for each site that ran; the spans are the sites
/// by number.
pub fn emit_profiled(
    program: &cpl_ir::Program,
    name: &str,
    target: &Target,
) -> (String, Vec<cpl_diag::Span>) {
    let program = prepared(program, target);
    let mut emitter = emit::Emitter::new(&program, target);
    emitter.sites = Some(Vec::new());
    emitter.devirt = devirt::Devirt::collect_unmerged(&program);
    emitter.counted = cpl_ir::sites::user_stmts(&program);
    emitter.entries = cpl_ir::sites::user_entries(&program);
    let (cpp, sites) = emitter.file_with_sites(name, None);
    let tally = PROFILE_TALLY.replace("SITES", &sites.len().max(1).to_string());
    // Before the first namespace, after every include and `#if` block, so
    // the tally never lands inside a platform-only block.
    let at = cpp.find("\nnamespace ").map_or(0, |at| at + 1);
    (format!("{}{tally}{}", &cpp[..at], &cpp[at..]), sites)
}

const PROFILE_TALLY: &str = r#"#include <chrono>
namespace cpl_prof {
static unsigned long long hits[SITES], nanos[SITES];
static long long last = -1;
static std::chrono::steady_clock::time_point since = std::chrono::steady_clock::now();
inline void charge(std::chrono::steady_clock::time_point now) {
    if (last >= 0) nanos[last] += std::chrono::duration_cast<std::chrono::nanoseconds>(now - since).count();
    since = now;
}
inline void mark(long long site) { charge(std::chrono::steady_clock::now()); last = site; ++hits[site]; }
// Opened by a user fn body: on return the time is the calling statement's again.
struct Frame {
    long long saved = last;
    ~Frame() { charge(std::chrono::steady_clock::now()); last = saved; }
};
struct Tally {
    ~Tally() {
        charge(std::chrono::steady_clock::now());
        const char* path = std::getenv("CPL_PROFILE");
        FILE* out = path ? std::fopen(path, "w") : stderr;
        if (!out) return;
        for (long long i = 0; i < SITES; ++i)
            if (hits[i]) std::fprintf(out, "%lld %llu %llu\n", i, hits[i], nanos[i]);
        if (out != stderr) std::fclose(out);
    }
} tally;
}
"#;

/// The file with two spaces a level, not the four it is written with.
fn narrow(cpp: &str) -> String {
    let mut narrowed = String::with_capacity(cpp.len());
    for line in cpp.split_inclusive('\n') {
        let code = line.trim_start_matches(' ');
        let indent = line.len() - code.len();
        narrowed.extend(std::iter::repeat_n(' ', indent / 2));
        narrowed.push_str(code);
    }
    narrowed
}

/// `cpp` without its comment lines, but for the header at the top of the
/// file when `header`. A comment after code stays: it is on a line of code.
fn uncommented(cpp: &str, header: bool) -> String {
    let mut leading = header;
    let mut out = String::with_capacity(cpp.len());
    for line in cpp.split_inclusive('\n') {
        let comment = line.trim_start().starts_with("//");
        leading &= comment;
        if leading || !comment {
            out.push_str(line);
        }
    }
    out
}

/// The C++ of [`emit`] made to fit a judge's source limit.
pub struct Fitted {
    pub cpp: String,
    /// How long the file is as written: with its comments and its source in the header.
    pub full: usize,
    /// Whether the comments on the C++ are kept: they are the first thing
    /// left out, as the source in the header says more to its reader.
    pub comments: bool,
    /// Whether the source is in the header: the last thing left out.
    pub header: bool,
    /// Whether each line is indented half as far: the second.
    pub narrow: bool,
    /// Whether each line is without indentation or a space the compiler
    /// does not need: the third.
    pub compact: bool,
    /// The shorter names given to the names codegen numbered: the fourth.
    pub names: std::collections::BTreeMap<String, String>,
    /// The instances of one generic fn made one C++ template, before the
    /// spaces go; for the one it stands in place of, the template.
    pub folded: std::collections::BTreeMap<String, Option<String>>,
}

impl Fitted {
    /// What was left out to fit, said to the contestant; `None` when nothing was.
    pub fn note(&self, file: &str, bar: &str, judge: &str) -> Option<String> {
        let mut left = Vec::new();
        if !self.comments {
            left.push("the comments on the C++ are left out");
        }
        if !self.header {
            left.push("the source in the header is left out");
        }
        if !self.folded.is_empty() {
            left.push("the instances of a generic fn that differ only in types and integers are one template");
        }
        if self.narrow && !self.compact {
            left.push("the lines are indented half as far");
        }
        if self.compact {
            left.push("the lines are without indentation or a space the compiler does not need");
        }
        if !self.names.is_empty() {
            left.push("the numbered names are shortened");
        }
        (!left.is_empty()).then(|| {
            format!(
                "cpl: as written, the C++ for `{file}` would be {} bytes, over the {bar} of judge `{judge}`; \
                 {}\n",
                self.full,
                left.join(", ")
            )
        })
    }
}

/// The C++17 file for `program` in at most `limit` bytes. What is for the
/// reader gives way to what the judge accepts: first the comments on the
/// C++, then half of its indentation, then the rest of it with every space
/// the compiler does not need, then the length of the names codegen
/// numbered, and last the source in the header, unless the file is over
/// `aim` without it too. `Err` is the length of the shortest file when even
/// that is over.
pub fn emit_within(
    program: &cpl_ir::Program,
    name: &str,
    source: &str,
    limit: usize,
) -> Result<Fitted, usize> {
    emit_within_for(program, name, source, limit, &Target::default())
}

/// [`emit_within`] for a judge.
pub fn emit_within_for(
    program: &cpl_ir::Program,
    name: &str,
    source: &str,
    limit: usize,
    target: &Target,
) -> Result<Fitted, usize> {
    emit_aiming(program, name, source, (limit, limit), target)
        .map_err(|shortest| shortest.cpp.len())
}

/// [`emit_within_for`] that gives way until the file is at most `aim`
/// bytes, or as short as it gets, and is `Err` with the shortest file only
/// when that is over `limit`.
pub fn emit_aiming(
    program: &cpl_ir::Program,
    name: &str,
    source: &str,
    (aim, limit): (usize, usize),
    target: &Target,
) -> Result<Fitted, Fitted> {
    let emit = |program, name, source| emit_for(program, name, source, target);
    let cpp = emit(program, name, Some(source));
    let full = cpp.len();
    let none = std::collections::BTreeMap::new;
    let nofold = std::collections::BTreeMap::new;
    let fitted = |cpp, comments, header, narrow, compact, names, folded| Fitted {
        cpp,
        full,
        comments,
        header,
        narrow,
        compact,
        names,
        folded,
    };
    if full <= aim {
        return Ok(fitted(cpp, true, true, false, false, none(), nofold()));
    }
    let cpp = uncommented(&cpp, true);
    if cpp.len() <= aim {
        return Ok(fitted(cpp, false, true, false, false, none(), nofold()));
    }
    let narrowed = narrow(&cpp);
    if narrowed.len() <= aim {
        return Ok(fitted(narrowed, false, true, true, false, none(), nofold()));
    }
    let origins = instance_origins(program, target);
    let fold::Folded { cpp, members } = fold::fold(&cpp, &origins);
    if !members.is_empty() {
        let narrowed = narrow(&cpp);
        if narrowed.len() <= aim {
            return Ok(fitted(narrowed, false, true, true, false, none(), members));
        }
    }
    let compacted = compact::compact(&cpp, true, &none());
    if compacted.len() <= aim {
        return Ok(fitted(compacted, false, true, true, true, none(), members));
    }
    let headed = compact::short_names(&compacted);
    let renamed = compact::compact(&cpp, true, &headed);
    if renamed.len() <= aim {
        return Ok(fitted(renamed, false, true, true, true, headed, members));
    }
    let bare = fold::fold(&uncommented(&emit(program, name, None), true), &origins);
    let names = compact::short_names(&compact::compact(&bare.cpp, true, &none()));
    let shortest = fitted(
        compact::compact(&bare.cpp, true, &names),
        false,
        false,
        true,
        true,
        names,
        bare.members,
    );
    if shortest.cpp.len() > aim && renamed.len() <= limit {
        return Ok(fitted(renamed, false, true, true, true, headed, members));
    }
    match shortest.cpp.len() <= limit {
        true => Ok(shortest),
        false => Err(shortest),
    }
}

/// Each fn's C++ name, and the generic fn it is an instance of, with the
/// number of a clone `split` made (`_k0`) dropped too.
fn instance_origins(
    program: &cpl_ir::Program,
    target: &Target,
) -> std::collections::BTreeMap<String, String> {
    let program = prepared(program, target);
    let clone = |segment: &str| {
        let mut s = segment;
        while let Some((head, n)) = s.rsplit_once("_k") {
            if n.is_empty() || !n.bytes().all(|c| c.is_ascii_digit()) {
                break;
            }
            s = head;
        }
        s.to_string()
    };
    program
        .fns
        .iter()
        .map(|f| {
            let origin = unmangled(&f.name);
            let origin: Vec<String> = origin.split('.').map(clone).collect();
            (names::fn_name(&f.name), origin.join("."))
        })
        .collect()
}

#[cfg(test)]
mod copy_tests;
#[cfg(test)]
mod devirt_tests;
#[cfg(test)]
mod fold_tests;
#[cfg(test)]
mod prune_tests;
#[cfg(test)]
mod range_tests;
#[cfg(test)]
mod sink_tests;
#[cfg(test)]
mod splice_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod walk_tests;
#[cfg(test)]
mod width_tests;
