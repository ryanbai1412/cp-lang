//! Lowering of a checked program to a single C++17 file.

mod devirt;
mod emit;
mod expr;
mod names;
mod niche;
mod ranges;
mod runtime;
mod scope;
mod show;
mod stmt;
mod unordered;
mod walk;

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
}

impl Default for Target {
    fn default() -> Target {
        Target { stamp: None, thread_stack: Some(1 << 30), windows: false }
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
/// up to `total`. `source` is the header's, if it kept it, and `narrowed`
/// whether its lines are indented half as far.
pub fn sizes(
    program: &cpl_ir::Program,
    name: &str,
    source: Option<&str>,
    target: &Target,
    fitted: (bool, bool),
    total: usize,
) -> Vec<Size> {
    let (comments, narrowed) = fitted;
    let mut program = program.clone();
    (program.unordered_maps, program.unordered_sets) = unordered::find(&program);
    let mut emitter = emit::Emitter::new(&program);
    emitter.target = target.clone();
    let (_, chunks) = emitter.file_with_chunks(name, source);
    let fit = |part: Part, text: &str| {
        let text = match comments || part == Part::Header {
            true => text.to_string(),
            false => uncommented(text, false),
        };
        match narrowed {
            true => narrow(&narrow(&text)).len(),
            false => narrow(&text).len(),
        }
    };
    let mut sizes: Vec<Size> = chunks
        .iter()
        .filter(|(_, _, text)| !text.is_empty())
        .map(|(part, name, text)| Size { part: *part, name: unmangled(name), bytes: fit(*part, text) })
        .filter(|size| size.bytes > 0)
        .collect();
    let named: usize = sizes.iter().map(|size| size.bytes).sum();
    sizes.push(Size { part: Part::Glue, name: "namespaces and main".into(), bytes: total.saturating_sub(named) });
    sizes
}

/// A fn's name as its source wrote it: `Rooted.path`, not the name of one
/// instance, `Rooted_D0_V_Int_7.path_Arr1_84`. A part of an instance's name
/// starts with `_` and a capital or a digit.
fn unmangled(name: &str) -> String {
    let cut = |segment: &str| {
        let bytes = segment.as_bytes();
        let at = (1..bytes.len())
            .find(|&at| bytes[at - 1] == b'_' && bytes[at].is_ascii_uppercase() | bytes[at].is_ascii_digit())
            .map_or(segment.len(), |at| at - 1);
        segment[..at].to_string()
    };
    name.split('.').map(cut).collect::<Vec<_>>().join(".")
}

/// [`emit`] for a judge.
pub fn emit_for(program: &cpl_ir::Program, name: &str, source: Option<&str>, target: &Target) -> String {
    let mut program = program.clone();
    (program.unordered_maps, program.unordered_sets) = unordered::find(&program);
    let mut emitter = emit::Emitter::new(&program);
    emitter.target = target.clone();
    narrow(&emitter.file(name, source))
}

/// [`emit_for`] with every statement counting its starts and the time until
/// the next one starts, the books `cpl_interp::profile` keeps. The tally is
/// written when the program exits, to the file named by `CPL_PROFILE`, a
/// line `site hits nanos` for each site that ran; the spans are the sites
/// by number.
pub fn emit_profiled(program: &cpl_ir::Program, name: &str, target: &Target) -> (String, Vec<cpl_diag::Span>) {
    let mut program = program.clone();
    (program.unordered_maps, program.unordered_sets) = unordered::find(&program);
    let mut emitter = emit::Emitter::new(&program);
    emitter.target = target.clone();
    emitter.sites = Some(Vec::new());
    emitter.counted = cpl_ir::sites::user_stmts(&program);
    let (cpp, sites) = emitter.file_with_sites(name, None);
    let tally = PROFILE_TALLY.replace("SITES", &sites.len().max(1).to_string());
    let head = cpp.find("\nnamespace ").unwrap_or(cpp.len());
    let at = cpp[..head]
        .rfind("#include")
        .and_then(|at| cpp[at..].find('\n').map(|end| at + end + 1))
        .unwrap_or(0);
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
    /// Whether the source is in the header: the second.
    pub header: bool,
    /// Whether each line is indented half as far: the third.
    pub narrow: bool,
}

impl Fitted {
    /// What was left out to fit, said to the contestant; `None` when nothing was.
    pub fn note(&self, file: &str, limit: usize, judge: &str) -> Option<String> {
        let mut left = Vec::new();
        if !self.comments {
            left.push("the comments on the C++ are left out");
        }
        if !self.header {
            left.push("the source in the header is left out");
        }
        if self.narrow {
            left.push("the lines are indented half as far");
        }
        (!left.is_empty()).then(|| {
            format!(
                "cpl: as written, the C++ for `{file}` would be {} bytes, over the {limit}-byte limit of judge \
                 `{judge}`; {}\n",
                self.full,
                left.join(", ")
            )
        })
    }
}

/// The C++17 file for `program` in at most `limit` bytes. What is for the
/// reader gives way to what the judge accepts: first the comments on the
/// C++, then the source in the header, then half of what indentation is left. `Err` is the length of
/// the shortest file when even that is over.
pub fn emit_within(program: &cpl_ir::Program, name: &str, source: &str, limit: usize) -> Result<Fitted, usize> {
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
    let emit = |program, name, source| emit_for(program, name, source, target);
    let cpp = emit(program, name, Some(source));
    let full = cpp.len();
    if full <= limit {
        return Ok(Fitted { cpp, full, comments: true, header: true, narrow: false });
    }
    let cpp = uncommented(&cpp, true);
    if cpp.len() <= limit {
        return Ok(Fitted { cpp, full, comments: false, header: true, narrow: false });
    }
    let cpp = uncommented(&emit(program, name, None), true);
    if cpp.len() <= limit {
        return Ok(Fitted { cpp, full, comments: false, header: false, narrow: false });
    }
    let narrowed = narrow(&cpp);
    match narrowed.len() <= limit {
        true => Ok(Fitted { cpp: narrowed, full, comments: false, header: false, narrow: true }),
        false => Err(narrowed.len()),
    }
}

#[cfg(test)]
mod tests;
