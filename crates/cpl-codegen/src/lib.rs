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
#[derive(Clone, Debug, Default)]
pub struct Target {
    /// A line for the header that names the judge.
    pub stamp: Option<String>,
    /// Whether the judge's stack holds deep recursion, so the program
    /// needs no stack of its own.
    pub deep_stack: bool,
}

/// [`emit`] for a judge.
pub fn emit_for(program: &cpl_ir::Program, name: &str, source: Option<&str>, target: &Target) -> String {
    let mut program = program.clone();
    (program.unordered_maps, program.unordered_sets) = unordered::find(&program);
    let mut emitter = emit::Emitter::new(&program);
    emitter.target = target.clone();
    narrow(&emitter.file(name, source))
}

/// [`emit`] with every statement counting its starts and the time until
/// the next one starts, the books `cpl_interp::profile` keeps. The tally is
/// written when the program exits, to the file named by `CPL_PROFILE`, a
/// line `site hits nanos` for each site that ran; the spans are the sites
/// by number.
pub fn emit_profiled(program: &cpl_ir::Program, name: &str) -> (String, Vec<cpl_diag::Span>) {
    let mut program = program.clone();
    (program.unordered_maps, program.unordered_sets) = unordered::find(&program);
    let mut emitter = emit::Emitter::new(&program);
    emitter.sites = Some(Vec::new());
    emitter.counted = cpl_ir::sites::user_stmts(&program);
    let (cpp, sites) = emitter.file_with_sites(name, None);
    let tally = PROFILE_TALLY.replace("SITES", &sites.len().max(1).to_string());
    let at = cpp
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

/// The C++ of [`emit`] made to fit a judge's source limit.
pub struct Fitted {
    pub cpp: String,
    /// How long the file is with its source in the header.
    pub full: usize,
    /// Whether the source is in the header: it is the first thing left out.
    pub header: bool,
    /// Whether each line is indented half as far: the second.
    pub narrow: bool,
}

/// The C++17 file for `program` in at most `limit` bytes. What is for the
/// reader gives way to what the judge accepts: first the source in the
/// header, then half of what indentation is left. `Err` is the length of
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
        return Ok(Fitted { cpp, full, header: true, narrow: false });
    }
    let cpp = emit(program, name, None);
    if cpp.len() <= limit {
        return Ok(Fitted { cpp, full, header: false, narrow: false });
    }
    let narrowed = narrow(&cpp);
    match narrowed.len() <= limit {
        true => Ok(Fitted { cpp: narrowed, full, header: false, narrow: true }),
        false => Err(narrowed.len()),
    }
}

#[cfg(test)]
mod tests;
