//! Messages syntax emits.

super::msgs! {
    COALESCE_MIXED = UNPARENTHESIZED_COALESCE, "`??` mixed with `+ - * / %` needs parentheses";
    C_LOGIC_OP = UNEXPECTED_TOKEN, "`{}` is `{}` in CP Lang";
    FIELD_AFTER_DOT = EXPECTED_NAME, "expected a field or method name after `.`";
    CPP_NEEDS_STRING = EXPECTED_EXPR, "expected a string literal in `@{}(...)`";
    LEADING_DOT_FLOAT = BAD_NUMBER, "`.{}` isn't a number; write a leading zero: `0.{}`";
    BARE_OP_TUPLE = BARE_OP_TUPLE, "`({}, ...)` is not an operation; write `each({}, ...)` or `Semiring(...)`";
    PAREN_STATEMENTS = REMOVED_SYNTAX, "statements in parentheses are written as a block, `{ a; b }`";
    DOUBLED_OPERATOR = EXPECTED_EXPR, "expected a value {}, found {}";
    FIELD_NEEDS_NAME = EXPECTED_EXPR, "a record field needs a name: write `name: value`";
    ELSE_IF = REMOVED_SYNTAX, "use `elif`, not `else if`";
    MATCH_NEEDS_ARMS = EXPECTED_BLOCK, "expected indented `pattern => value` arms after `match`";
    BAD_NUMBER_FORMAT = UNEXPECTED_TOKEN, "`:{}` is not a number format";
    OPTIONAL_MARK_FIRST = EXPECTED_TYPE, "an optional type puts its `?` first in CP Lang";
    STR_LEN_LABEL = EXPECTED_TYPE, "a `Str(…)` type takes the range of its length, `Str(len: 1..=n)`";
    STR_LEN_RANGE = EXPECTED_TYPE, "the length of a `Str(len: …)` is a range, such as `1..=n`";
    RAGGED_READ = EXPECTED_TYPE, "a ragged read `[i in V][..i]T` is not a type";
    LIST_TYPE_BRACKETS = REMOVED_SYNTAX, "a list type is `List<{}>`, not `[{}]`";
    DYN_TRAIT_OBJECT = UNSUPPORTED, "CP Lang has no `dyn` trait objects; use a generic type parameter with a trait bound instead";
    TYPE_SPELLING = EXPECTED_TYPE, "CP Lang spells the type `{}`, not `{}`";
    AMP_TYPE = EXPECTED_TYPE, "CP Lang has no `&` types";
    TYPE_AFTER_COLON = EXPECTED_TYPE, "expected a type after `:`, found {}";
    EXPECTED_A_TYPE = EXPECTED_TYPE, "expected a type";
    TYPE_ARGS_IN_ANGLES = EXPECTED_TYPE, "CP Lang writes type arguments in `<>`: `{}<...>`, not `{}[...]`";
}
