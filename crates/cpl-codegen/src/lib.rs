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
    emit::Emitter::new(&program).file(name, source)
}

#[cfg(test)]
mod tests;
