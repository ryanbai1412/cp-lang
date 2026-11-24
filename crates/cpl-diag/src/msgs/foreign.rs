//! Messages foreign emits.

super::msgs! {
    LET_KEYWORD = REMOVED_SYNTAX, "CP Lang declares a variable with `let`, not `{}`";
    C_DECL_TYPE = REMOVED_SYNTAX, "CP Lang writes a variable's type after its name, not `{}` before it";
    COLON_RETURN_TYPE = EXPECTED_TOKEN, "a return type follows `->` in CP Lang, not `:`";
    RUST_PRINT = EXPECTED_TOKEN, "CP Lang prints with `print(...)`, not `{}!`";
    RUST_REPEAT = REMOVED_SYNTAX, "Rust's `[x; n]` is a fill here: `[n].filled(x)`";
    EMPTY_MAP = EXPECTED_TOKEN, "an empty map is `Map<K, V>()` in CP Lang, or `[:]` where its type is known";
    FOREIGN_NEW = REMOVED_SYNTAX, "CP Lang has no `new`: a struct is built by calling its type";
    C_DEREF = REMOVED_SYNTAX, "CP Lang has no pointers: a value is used as it is, without `*`";
    C_ADDRESS = REMOVED_SYNTAX, "CP Lang has no pointers: a value is passed as it is, without `&`";
    C_REF_ARG = REMOVED_SYNTAX, "CP Lang marks an argument the fn changes with `mut`, not `&`";
    PYTHON_SPLAT = REMOVED_SYNTAX, "CP Lang has no `*` unpacking: a list is passed as it is";
    C_CAST = REMOVED_SYNTAX, "CP Lang has no C casts";
    STRUCT_LITERAL = EXPECTED_TOKEN, "CP Lang builds a struct by calling its type, not with `{{ ... }}`";
    LIST_PATTERN = EXPECTED_PATTERN, "a list can't be unpacked in CP Lang: only tuples can";
    FOREIGN_STEP = EXPECTED_TOKEN, "a range's stride is `by` in CP Lang, not `step`";
    CPP_NPOS = REMOVED_SYNTAX, "CP Lang has no `npos`: `find` gives `None` when there is no match";
    C_TERNARY = REMOVED_SYNTAX, "CP Lang has no `c ? a : b`";
    CPP_VALUE = REMOVED_SYNTAX, "`{}` is C++; CP Lang writes the value as `{}`";
    REVERSED_SLICE = EXPECTED_TOKEN, "CP Lang reverses with `.rev`, not `[::-1]`";
}
