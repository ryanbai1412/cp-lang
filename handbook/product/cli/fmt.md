# Format

> 25 minutes in, you pasted a loop from your last solution, renamed two
> variables and nested it one level deeper. The indentation is ragged and
> one call runs past the edge of the screen. You don't want to think about it.

You save. The editor runs `cpl fmt` on the file: four spaces per level, one
space around binary operators, the long call broken one argument per line
with a trailing comma, consecutive trailing comments aligned. Nothing you
wrote changes meaning, and nothing you didn't touch moves: a one-line edit
reformats that line. Typing a newline tidies the line you just finished;
selecting a block and formatting it touches only those lines. From the
shell, `cpl fmt .` rewrites every `.cp` file in the project, and
`cpl fmt --check` lists the files that would change without writing, for a
pre-commit hook.

**Off the happy path.** The file doesn't parse yet: `cpl fmt` still formats
what it can and never fuses two tokens; a file the lexer can't read is
refused with the error, and the others in the run are formatted. You want
two spaces per level or 80 columns: `[fmt]` in `cp.toml` sets
`indent_width` and `line_width`, the only two knobs. A Windows checkout
keeps its CRLF line ends. `use` lines are sorted, std first, unless sorting
would change what a line means.

**Ideal against today.** The ideal is never thinking about layout: one
style, applied on save, idempotent, stable under small edits, identical in
the editor, the CLI and the playground. Today all of that holds and is
tested over the whole corpus and every doc snippet. Gap: no test pins the
playground's formatting to `cpl fmt`.

**Against the usual setup.** C++ contestants run clang-format with a
hand-tuned `.clang-format`, or nothing; gofmt and rustfmt showed that one
style with almost no options ends layout arguments. `cpl fmt` takes that
side: two width knobs, everything else fixed.

**Dimensions.** Input state (clean, ragged, doesn't parse, can't lex), scope
(file, directory, selection, the line just typed, stdin), trigger (save,
newline, command, hook), style (default, `[fmt]` widths), line ends (LF,
CRLF), and output (rewrite, check, JSON).
