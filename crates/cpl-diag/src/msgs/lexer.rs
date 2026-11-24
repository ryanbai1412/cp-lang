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
    UNUSED_CHAR = UNKNOWN_CHAR, "`{}` isn't used in CP Lang";
    /// The literal, then its suffix.
    C_INT_SUFFIX = BAD_NUMBER, "`{}` isn't a number: CP Lang has no `{}` suffix";
    /// The literal, then the digit.
    NOT_A_BINARY_DIGIT = BAD_NUMBER, "`{}` isn't a number: `{}` isn't a binary digit";
    NOT_A_NUMBER = BAD_NUMBER, "`{}` isn't a number";
    NO_INCREMENT = UNKNOWN_CHAR, "`++` doesn't increment in CP Lang";
    NO_DECREMENT = UNKNOWN_CHAR, "`--` doesn't decrement in CP Lang";
    /// Ours (`fn`), then the foreign word (`def`).
    FOREIGN_FN = REMOVED_SYNTAX, "CP Lang declares a function with `{}`, not `{}`";
    FOREIGN_USE = REMOVED_SYNTAX, "CP Lang imports a module with `{}`, not `{}`";
    FOREIGN_MATCH = REMOVED_SYNTAX, "CP Lang matches a value with `{}`, not `{}`";
    STRING_NEVER_CLOSED = UNTERMINATED_STRING, "this string is never closed";
    /// The character, then the literal's type (`Str`, `Char`).
    NON_ASCII_LITERAL = NON_ASCII, "`{}` is not ASCII; a {} literal holds single-byte ASCII characters only";
    BAD_CHAR_LITERAL = BAD_CHAR_LITERAL, "a character is written `'a'`, with exactly one character between the quotes";
    TAB_INDENT = TAB_INDENT, "this line is indented with a tab; cp indents with spaces only";
}
