//! Messages stmt emits.

super::msgs! {
    STRAY_INDENT = UNEXPECTED_INDENT, "this line is indented, but no block starts above it";
    USE_DIRECTIVE = REMOVED_SYNTAX, "use `{}` instead of `{}`";
    SEMI_AFTER_DECORATOR = UNEXPECTED_TOKEN, "no `;` after a decorator";
    UNKNOWN_DIRECTIVE = UNEXPECTED_TOKEN, "unknown directive `@!{}`";
    UNKNOWN_DECORATOR = UNEXPECTED_TOKEN, "unknown decorator `@{}`";
    DIRECTIVE_IN_BLOCK = UNEXPECTED_TOKEN, "`@!{}` stands at the top level of the file";
    INTERACTIVE_ARGS = UNEXPECTED_TOKEN, "`@!interactive` takes nothing";
    ALLOW_REMOVED = REMOVED_SYNTAX, "`@allow` was removed in CP Lang 0.3; lints are contexts";
    MEMO_KIND_REMOVED = REMOVED_SYNTAX, "`@memo({})` was removed in CP Lang 0.2.2";
    MULTITEST_COUNT_REMOVED = REMOVED_SYNTAX, "`@multitest(count:)` was removed in CP Lang 0.2.2";
    ALT_MAIN_REMOVED = REMOVED_SYNTAX, "`{}` was removed in CP Lang 0.3; write `@alt(program)` or `@alt(f)`";
    CHECKER_REMOVED = REMOVED_SYNTAX, "`@checker` was removed in CP Lang 0.3; write `@check`";
    GRADER_CONST_REMOVED = REMOVED_SYNTAX, "`@grader const` was removed in CP Lang 0.2.2";
    ELLIPSIS = UNEXPECTED_TOKEN, "`...` is not an expression; write `todo` for an unwritten part";
    NO_OPENER = UNMATCHED_CLOSER, "this {} has no matching opener";
    EXPECTED_STMT_END = UNEXPECTED_TOKEN, "expected the end of the statement, found {}";
    NO_MACROS = UNSUPPORTED, "CP Lang has no macros; use a generic fn or inline helper instead";
    UNALIGNED_ELSE = UNEXPECTED_TOKEN, "this `{}` has no {} above it at the same indentation";
    KEYWORD_REMOVED = REMOVED_SYNTAX, "`{}` was removed in CP Lang 0.2.2";
    MUT_FN = EXPECTED_EXPR, "a fn is not marked `mut`: mark the parameter it changes, `fn f(mut x: T)`";
    PURE_FN = UNEXPECTED_TOKEN, "`pure fn` was removed; write `fn`";
    LET_MUT = UNEXPECTED_TOKEN, "`let mut` is `var` in CP Lang";
    TYPED_NO_VALUE = EXPECTED_TOKEN, "`{}` with a type and no value: write `=` and the value it starts with (R01.10.2)";
    WHERE_GUARD = REMOVED_SYNTAX, "`where` guards were removed in CP Lang 0.2.2";
    FOR_MUT = REMOVED_SYNTAX, "`for mut` was removed in CP Lang 0.2.2";
    LOOP_WHERE = REMOVED_SYNTAX, "loop filters use `if`, not `where`";
    BINDER_NEEDS_LET = UNEXPECTED_TOKEN, "a binder after `,` needs its own `let`";
    IF_LET_WHERE = REMOVED_SYNTAX, "use `and`, not `where`, after `if let`";
    EXPECTED_COLON = EXPECTED_TOKEN, "expected `:` after {}, found {}";
    EXPECTED_BLOCK_AFTER = EXPECTED_BLOCK, "expected an indented block after {}";
}
