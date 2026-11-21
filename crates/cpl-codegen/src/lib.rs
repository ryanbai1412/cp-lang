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
    let mut program = program.clone();
    (program.unordered_maps, program.unordered_sets) = unordered::find(&program);
    narrow(&emit::Emitter::new(&program).file(name, source))
}

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
pub fn emit_within(
    program: &cpl_ir::Program,
    name: &str,
    source: &str,
    limit: usize,
) -> Result<Fitted, usize> {
    let cpp = emit(program, name, Some(source));
    let full = cpp.len();
    if full <= limit {
        return Ok(Fitted {
            cpp,
            full,
            header: true,
            narrow: false,
        });
    }
    let cpp = emit(program, name, None);
    if cpp.len() <= limit {
        return Ok(Fitted {
            cpp,
            full,
            header: false,
            narrow: false,
        });
    }
    let narrowed = narrow(&cpp);
    match narrowed.len() <= limit {
        true => Ok(Fitted {
            cpp: narrowed,
            full,
            header: false,
            narrow: true,
        }),
        false => Err(narrowed.len()),
    }
}

#[cfg(test)]
mod tests;
