//! Messages lexer emits.

super::msgs! {
    LONE_CARRIAGE_RETURN = UNEXPECTED_TOKEN, "a carriage return that does not start a `\\r\\n` line end";
    FLOOR_DIV_IS_COMMENT = UNEXPECTED_TOKEN, "`//` starts a comment in CP Lang, so the rest of this line is skipped";
    CPP_FAST_IO = REMOVED_SYNTAX, "C++'s fast-I/O setup: CP Lang's `read()` and `print` are already fast";
    PY_FAST_INPUT = REMOVED_SYNTAX, "Python's fast-input setup: CP Lang's `read()` is already fast";
    /// The foreign line's keyword: `#include`, `import`, ...
    NO_IMPORT = REMOVED_SYNTAX, "CP Lang has no `{}`: the prelude is always in scope";
    NO_MAIN = REMOVED_SYNTAX, "CP Lang has no `main`: the file's top-level lines are the program";
    /// The line's width, then the enclosing widths joined by ` or `.
    INDENT_MATCHES_NO_BLOCK = BAD_DEDENT, "this line is indented {} spaces, which matches no enclosing block ({})";
    /// The opening delimiter.
    NEVER_CLOSED = UNCLOSED_DELIMITER, "this `{}` is never closed";
    FLOAT_NEEDS_LEADING_DIGIT = BAD_NUMBER, "`{}` isn't a number: a float needs a digit before the point";
    /// Ours (`==`), then theirs (`===`).
    STRICT_EQUALITY = UNKNOWN_CHAR, "CP Lang compares with `{}`, not `{}`";
    CONCAT_PLUS_PLUS = UNKNOWN_CHAR, "CP Lang concatenates with `+`";
}
