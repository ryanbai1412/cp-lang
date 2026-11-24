//! Habits from other languages, as data. Each row is a foreign spelling the
//! front end meets, the help it gives, and a mistake program that shows it:
//! `crates/cpl-cli/tests/diagnostics.rs` runs every row, so a row's help is
//! seen where its mistake is written and `cpl fix` leaves the mistake
//! checking, unless the row says why no one edit is right.

use crate::{codes, Code};
use std::fmt::{Display, Write};

/// Where the foreign spelling stands: called, `stoi(s)`; as a value, `True`;
/// or a shape the front end recognises, `for (int i = 0; i < n; i++)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Call,
    Value,
    Shape,
}

/// One foreign habit. Its mistake program is
/// `crates/cpl-cli/tests/habits/<key>.cp`, the error's line marked `// <- here`.
#[derive(Debug)]
pub struct Habit {
    pub key: &'static str,
    pub kind: Kind,
    /// The spellings that call for this help: `stoi`, `atoi`, `int`.
    pub names: &'static [&'static str],
    /// Where the habit comes from: `C++`, `Python`.
    pub lang: &'static str,
    pub code: Code,
    /// The help, or for a shape its help or fix title, with `{}` holes.
    pub help: &'static str,
    /// What `cpl fix` writes for the name, when one word replaces it.
    pub fix: &'static str,
    /// Why `cpl fix` can't mend the mistake, or empty when it does.
    pub no_fix: &'static str,
}

impl Habit {
    /// The help with its `{}` holes filled by `args` in order; `{{` and `}}` are braces.
    pub fn say(&self, args: &[&dyn Display]) -> String {
        let mut out = String::new();
        let mut args = args.iter();
        let mut rest = self.help;
        while let Some(i) = rest.find(['{', '}']) {
            out.push_str(&rest[..i]);
            let tail = &rest[i..];
            let step = if tail.starts_with("{}") {
                if let Some(arg) = args.next() {
                    let _ = write!(out, "{arg}");
                }
                2
            } else {
                out.push_str(&tail[..1]);
                if tail.starts_with("{{") || tail.starts_with("}}") {
                    2
                } else {
                    1
                }
            };
            rest = &tail[step..];
        }
        out.push_str(rest);
        out
    }
}

/// The habit the unknown name `name`, written as `kind`, is, if any.
pub fn find(kind: Kind, name: &str) -> Option<&'static Habit> {
    HABITS
        .iter()
        .find(|h| h.kind == kind && h.names.contains(&name))
}

/// The help for the unknown name `name` written as `kind`, if it is a habit.
pub fn help(kind: Kind, name: &str) -> Option<&'static str> {
    find(kind, name).map(|h| h.help)
}

/// Every row: the named ones, then the shapes.
pub fn all() -> impl Iterator<Item = &'static Habit> {
    HABITS.iter().chain(SHAPES.iter().copied())
}

const fn call(
    key: &'static str,
    names: &'static [&'static str],
    lang: &'static str,
    help: &'static str,
    no_fix: &'static str,
) -> Habit {
    Habit {
        key,
        kind: Kind::Call,
        names,
        lang,
        code: codes::UNRESOLVED_NAME,
        help,
        fix: "",
        no_fix,
    }
}

const fn value(
    key: &'static str,
    names: &'static [&'static str],
    lang: &'static str,
    help: &'static str,
    fix: &'static str,
) -> Habit {
    Habit {
        kind: Kind::Value,
        fix,
        ..call(key, names, lang, help, "")
    }
}

const fn shape(key: &'static str, code: Code, help: &'static str, no_fix: &'static str) -> Habit {
    Habit {
        key,
        kind: Kind::Shape,
        names: &[],
        lang: "",
        code,
        help,
        fix: "",
        no_fix,
    }
}

const ARGS_VARY: &str = "the call's arguments are another language's, so no one rewrite fits";

pub const HABITS: &[Habit] = &[
    call(
        "str",
        &["str", "to_string", "toString"],
        "C++/Python/Java",
        "make a string by interpolation: `\"{x}\"`",
        ARGS_VARY,
    ),
    call(
        "int",
        &["int", "stoi", "atoi", "parseInt"],
        "Python/C++/Java",
        "read a number from a string with `s.parse<Int>()!`",
        ARGS_VARY,
    ),
    call(
        "float",
        &["float", "stod", "parseFloat"],
        "Python/C++/Java",
        "read a number from a string with `s.parse<F64>()!`",
        ARGS_VARY,
    ),
    call(
        "input",
        &["input", "cin", "scanf"],
        "Python/C/C++",
        "read the input with `read`: `let n: Int = read()`",
        "a read is a declaration with its type, which the call doesn't say",
    ),
    call(
        "cout",
        &["cout", "printf", "puts"],
        "C/C++",
        "write the output with `print`: `print(a, b)`",
        "a format string or stream has no one `print` it maps to",
    ),
    call(
        "max_element",
        &["max_element", "min_element"],
        "C++",
        "a list's largest and smallest values are `a.max()!` and `a.min()!`",
        "C++ takes an iterator pair and returns an iterator",
    ),
    call(
        "accumulate",
        &["accumulate"],
        "C++",
        "a list's sum is `a.sum()`",
        "C++ takes an iterator pair and a start value",
    ),
    call(
        "builtin_popcount",
        &["__builtin_popcount", "__builtin_popcountll"],
        "C++",
        "the 1 bits are counted with `popcount(x)`",
        ARGS_VARY,
    ),
    call(
        "lower_bound",
        &["lower_bound", "upper_bound", "bisect_left", "bisect_right"],
        "C++/Python",
        "a sorted list is searched with `a.first_ge(x)` and `a.first_gt(x)`: the index, or `None` past the end",
        "the result is an index, where C++'s is an iterator",
    ),
    call(
        "memset",
        &["memset"],
        "C/C++",
        "a list is filled with its `fill`: `a.fill(0)`",
        "`memset`'s byte count has no meaning here",
    ),
    call(
        "make_pair",
        &["make_pair", "make_tuple"],
        "C++",
        "a tuple is written in parentheses: `(a, b)`",
        ARGS_VARY,
    ),
    call(
        "endl",
        &["endl"],
        "C++",
        "`print` ends the line itself: `print(a, b)` writes `a b` and a newline",
        "`endl` as a value has nothing to become",
    ),
    Habit { fix: "Int", ..call(
        "ord",
        &["ord"],
        "Python",
        "a character's code is `Int(c)`",
        "",
    ) },
    Habit { fix: "Char", ..call(
        "chr",
        &["chr"],
        "Python",
        "the character with code `i` is `Char(i)`",
        "",
    ) },
    call(
        "dict",
        &["dict"],
        "Python",
        "an empty map is `Map<K, V>()`, or `[:]` where its type is known",
        "a map's key and value types aren't in the call",
    ),
    call(
        "sys",
        &["sys"],
        "Python",
        "CP Lang has no `sys`: `read()` is already fast, so remove the `sys` line",
        "the line goes with what reads it, which is the user's call",
    ),
    call(
        "math",
        &["math", "Math"],
        "Python/Java",
        "math functions are methods here: `x.sqrt`, `x.abs`, `x.isqrt`",
        "not every math function has a method of the same name",
    ),
    value("true", &["True", "TRUE"], "Python/C", "CP Lang spells it `true`", "true"),
    value("false", &["False", "FALSE"], "Python/C", "CP Lang spells it `false`", "false"),
    value("null", &["null", "nil", "none", "NULL", "nullptr"], "C/C++/Java/Go/Python", "CP Lang spells it `None`", "None"),
    value("int_max", &["INT_MAX", "LLONG_MAX", "LONG_MAX", "INF"], "C/C++", "CP Lang spells it `inf`", "inf"),
    value("int_min", &["INT_MIN", "LLONG_MIN", "LONG_MIN"], "C/C++", "CP Lang spells it `-inf`", "-inf"),
];

pub static ARROW_LAMBDA: Habit = shape(
    "arrow_lambda",
    codes::EXPECTED_TOKEN,
    "`->` names a fn's return type; a lambda's body follows `=>`",
    "",
);
pub static ARROW_LAMBDA_2: Habit = shape("arrow_lambda_2", codes::EXPECTED_TOKEN, "write `=>`", "");
pub static WRITE: Habit = shape("write", codes::PARENS_MEAN_WORK, "write `{}`", "");
pub static BAD_OPERANDS_2: Habit = shape(
    "bad_operands_2",
    codes::BAD_OPERANDS,
    "put the value in the string: `\"{{x}}\"`",
    "which value goes where in the string is the writer's call",
);
pub static BAD_OPERANDS_3: Habit = shape(
    "bad_operands_3",
    codes::BAD_OPERANDS,
    "two Bools differ by `!=`",
    "",
);
pub static BARE_TUPLE_PATTERN: Habit = shape(
    "bare_tuple_pattern",
    codes::UNKNOWN_FIELD,
    "write `({})`",
    "",
);
pub static CHANGES: Habit = shape(
    "changes",
    codes::REMOVED_SYNTAX,
    "write `mut {}`: the fn gets the variable itself",
    "",
);
pub static CHAR_LITERAL_FIX: Habit = shape(
    "char_literal_fix",
    codes::TYPE_MISMATCH,
    "a character is written in single quotes: `{}`",
    "",
);
pub static CMP_TOKEN: Habit = shape(
    "cmp_token",
    codes::UNEXPECTED_TOKEN,
    "write `not in` instead of `!in`",
    "",
);
pub static CMP_TOKEN_2: Habit = shape(
    "cmp_token_2",
    codes::UNEXPECTED_TOKEN,
    "replace with `not `",
    "",
);
pub static CMP_TOKEN_3: Habit = shape(
    "cmp_token_3",
    codes::UNEXPECTED_TOKEN,
    "write `x {} None`; values are compared, there is no identity",
    "",
);
pub static COLON_RETURN_TYPE: Habit = shape(
    "colon_return_type",
    codes::EXPECTED_TOKEN,
    "the `:` becomes `->`: `-> {}`",
    "",
);
pub static COLON_RETURN_TYPE_2: Habit = shape(
    "colon_return_type_2",
    codes::EXPECTED_TOKEN,
    "write `->`",
    "",
);
pub static CPP_NPOS: Habit = shape(
    "cpp_npos",
    codes::REMOVED_SYNTAX,
    "compare with `None`: `s.find(t) != None`, or write `t in s`",
    "",
);
pub static CPP_NPOS_2: Habit = shape("cpp_npos_2", codes::REMOVED_SYNTAX, "write `None`", "");
pub static C_ADDRESS: Habit = shape(
    "c_address",
    codes::REMOVED_SYNTAX,
    "drop the `&`; a fn that changes an argument takes it as `mut x`",
    "",
);
pub static C_AUTO: Habit = shape("c_auto", codes::REMOVED_SYNTAX, "`let x = 0` for a value that stays, `var x = 0` for one that changes; the type comes from the value", "");
pub static C_AUTO_2: Habit = shape("c_auto_2", codes::REMOVED_SYNTAX, "write `let`", "");
pub static C_CASE: Habit = shape(
    "c_case",
    codes::REMOVED_SYNTAX,
    "write the pattern itself: `{}`",
    "",
);
pub static C_CASE_2: Habit = shape("c_case_2", codes::UNKNOWN_CHAR, "remove `{}`", "");
pub static C_COUNTDOWN_REPORT: Habit = shape(
    "c_countdown_report",
    codes::REMOVED_SYNTAX,
    "loop over a range: `for _ in 0..{}:`",
    "",
);
pub static C_COUNTDOWN_REPORT_2: Habit = shape(
    "c_countdown_report_2",
    codes::REMOVED_SYNTAX,
    "the body reads `{}`: write `while {} > 0:` and start the body with `{} -= 1`",
    "the body reads the counter, so how it counts down is the writer's call",
);
pub static C_DECL_ERROR: Habit = shape(
    "c_decl_error",
    codes::REMOVED_SYNTAX,
    "`let x: Int = 0` for a value that stays, `var x: Int = 0` for one that changes",
    "",
);
pub static C_DEREF: Habit = shape("c_deref", codes::REMOVED_SYNTAX, "drop the `*`", "");
pub static C_FOR: Habit = shape(
    "c_for",
    codes::REMOVED_SYNTAX,
    "loop over a range: `for i in 0..n:`",
    "",
);
pub static C_FOR_2: Habit = shape("c_for_2", codes::REMOVED_SYNTAX, "loop over the range", "");
pub static C_RANGE_FOR: Habit = shape(
    "c_range_for",
    codes::REMOVED_SYNTAX,
    "write `for {} in {}:`",
    "",
);
pub static C_RANGE_FOR_2: Habit = shape("c_range_for_2", codes::REMOVED_SYNTAX, "loop over it", "");
pub static C_RANGE_FOR_3: Habit = shape("c_range_for_3", codes::REMOVED_SYNTAX, "the loop variable is a copy, so change the list through its indexes: `for i in 0..{}.len: {}[i] += 1`", "");
pub static C_TERNARY: Habit = shape(
    "c_ternary",
    codes::REMOVED_SYNTAX,
    "write `if c: a else: b`",
    "",
);
pub static C_TERNARY_2: Habit = shape(
    "c_ternary_2",
    codes::REMOVED_SYNTAX,
    "write it as an inline `if`",
    "",
);
pub static DUPLICATE_TAG: Habit = shape(
    "duplicate_tag",
    codes::DUPLICATE_VARIANT_TAG,
    "input names one variant per tag: give one of them another `@tag`",
    "which variant gets another tag is the writer's call",
);
pub static EMPTY_BRACES: Habit = shape(
    "empty_braces",
    codes::TYPE_MISMATCH,
    "an empty {} is `{}`",
    "",
);
pub static EMPTY_TYPE_ARGS: Habit = shape(
    "empty_type_args",
    codes::REMOVED_SYNTAX,
    "leave the `<>` off: CP Lang infers type arguments",
    "",
);
pub static EMPTY_TYPE_ARGS_2: Habit = shape(
    "empty_type_args_2",
    codes::REMOVED_SYNTAX,
    "remove `<>`",
    "",
);
pub static ENUM_TAGS: Habit = shape(
    "enum_tags",
    codes::REMOVED_SYNTAX,
    "write `enum {} = {}`",
    "each tag moves onto its own variant, a rewrite of the whole declaration",
);
pub static FOREIGN_NEW: Habit = shape(
    "foreign_new",
    codes::REMOVED_SYNTAX,
    "drop `new`: `{}(...)`",
    "",
);
pub static FOREIGN_NEW_2: Habit = shape("foreign_new_2", codes::REMOVED_SYNTAX, "drop `new`", "");
pub static FOREIGN_STEP: Habit = shape(
    "foreign_step",
    codes::EXPECTED_TOKEN,
    "write `0..10 by 2`",
    "",
);
pub static FOREIGN_STEP_2: Habit =
    shape("foreign_step_2", codes::UNKNOWN_ARG_NAME, "write `by`", "");
pub static FOREIGN_THEN: Habit = shape(
    "foreign_then",
    codes::EXPECTED_TOKEN,
    "the condition stays; `{}` becomes `:`",
    "",
);
pub static FOREIGN_THEN_2: Habit = shape("foreign_then_2", codes::EXPECTED_TOKEN, "write `:`", "");
pub static FOR_OF: Habit = shape(
    "for_of",
    codes::EXPECTED_TOKEN,
    "replace `of` with `in`",
    "",
);
pub static F_STRING_PREFIX: Habit = shape(
    "f_string_prefix",
    codes::EXPECTED_TOKEN,
    "write `\"n = {{n}}\"`",
    "",
);
pub static F_STRING_PREFIX_2: Habit = shape(
    "f_string_prefix_2",
    codes::EXPECTED_TOKEN,
    "remove the `f`",
    "",
);
pub static GETS: Habit = shape("gets", codes::REMOVED_SYNTAX, "write `mut`", "");
pub static INDEX_ITEM: Habit = shape(
    "index_item",
    codes::EXPECTED_TOKEN,
    "write `xs[a..b]`; `xs[a..]` and `xs[..b]` leave out an end",
    "",
);
pub static INDEX_ITEM_2: Habit = shape(
    "index_item_2",
    codes::EXPECTED_TOKEN,
    "replace `:` with `..`",
    "",
);
pub static ITER_CALL_HELP: Habit = shape(
    "iter_call_help",
    codes::UNKNOWN_FIELD,
    "a list or string is iterated as it is: `for x in xs`",
    "",
);
pub static ITER_CALL_HELP_2: Habit = shape(
    "iter_call_help_2",
    codes::UNKNOWN_FIELD,
    "remove `.{}()`",
    "",
);
pub static KEYED_FIND: Habit = shape(
    "keyed_find",
    codes::UNKNOWN_FIELD,
    "CP Lang has no iterators: write `{}`",
    "",
);
pub static KEYWORD_ARG: Habit = shape(
    "keyword_arg",
    codes::EXPECTED_TOKEN,
    "write `name: value`, as in `print(xs, sep: \", \")`",
    "",
);
pub static LAZY_LEN: Habit = shape(
    "lazy_len",
    codes::MISSING_IMPL,
    "a lazy sequence has no stored length: `.count()` walks it once",
    "",
);
pub static LAZY_LEN_2: Habit = shape("lazy_len_2", codes::MISSING_IMPL, "write `.count()`", "");
pub static LIST_IN: Habit = shape("list_in", codes::MISSING_IMPL, "`in` is for sets, maps and ranges, where it is fast; a list is scanned in O(n) with `xs.contains(x)`", "");
pub static LIST_IN_2: Habit = shape("list_in_2", codes::MISSING_IMPL, "scan the list: `{}`", "");
pub static LIST_PATTERN: Habit = shape(
    "list_pattern",
    codes::EXPECTED_PATTERN,
    "index it, `let a = xs[0]`, or unpack a tuple: `let (a, b) = (xs[0], xs[1])`",
    "indexing and a tuple are both right; the writer picks",
);
pub static LOOP_ADVICE: Habit = shape(
    "loop_advice",
    codes::TYPE_MISMATCH,
    "for each value and its count, walk `runs(..){}`",
    "",
);
pub static MEMBER_ADVICE: Habit = shape("member_advice", codes::REMOVED_SYNTAX, "a domain reads `D.count`, `x.ord` and `D.nth(k)`, which traps outside `0..D.count`; `D.try_at(k)` gives an optional (R04.2.26)", "");
pub static MEMBER_ADVICE_2: Habit =
    shape("member_advice_2", codes::REMOVED_SYNTAX, "write `cuts`", "");
pub static MEMBER_ADVICE_3: Habit = shape(
    "member_advice_3",
    codes::UNKNOWN_FIELD,
    "the parts of this tuple are `.0` to `.{}`",
    "",
);
pub static MEMBER_ADVICE_4: Habit = shape(
    "member_advice_4",
    codes::UNKNOWN_FIELD,
    "change to `.{}`",
    "",
);
pub static MEMBER_FOR_CALL: Habit = shape(
    "member_for_call",
    codes::UNRESOLVED_NAME,
    "math functions are members or operators here: `{}`",
    "",
);
pub static MEMBER_FOR_CALL_2: Habit = shape("member_for_call_2", codes::UNRESOLVED_NAME, "CP Lang reads numbers by token, not by line: `let (n, m): (Int, Int) = read()` reads two, `read<[n]Int>()` reads n of them", "a line read becomes token reads whose types the call doesn't say");
pub static MEMBER_FOR_CALL_3: Habit = shape(
    "member_for_call_3",
    codes::UNRESOLVED_NAME,
    "{} in CP Lang: `{}`",
    "",
);
pub static NO_OPERATOR_IMPL: Habit = shape(
    "no_operator_impl",
    codes::TYPE_MISMATCH,
    "call it to get the {}: `{}()`",
    "",
);
pub static NO_OPERATOR_IMPL_2: Habit = shape(
    "no_operator_impl_2",
    codes::TYPE_MISMATCH,
    "call the method with `()` where `{}` is made",
    "the `()` goes where the method is made, another line than the error",
);
pub static NO_OPERATOR_IMPL_3: Habit = shape(
    "no_operator_impl_3",
    codes::MISSING_IMPL,
    "write `impl {}<{}> for {}` to define `{} {} {}`",
    "the impl's body is the writer's to write",
);
pub static OLD_OPTIONAL_SPELLING: Habit = shape(
    "old_optional_spelling",
    codes::REMOVED_SYNTAX,
    "optionals are `Some(x)` and `None`; `cpl migrate` rewrites every `{}` in a folder",
    "",
);
pub static PAREN_FOR: Habit = shape(
    "paren_for",
    codes::REMOVED_SYNTAX,
    "write `for x in xs:`",
    "",
);
pub static PYTHON_SPLAT: Habit = shape(
    "python_splat",
    codes::REMOVED_SYNTAX,
    "`print(a)` prints a list's items separated by spaces",
    "",
);
pub static PYTHON_SPLAT_2: Habit = shape(
    "python_splat_2",
    codes::REMOVED_SYNTAX,
    "pass the list itself, or its items one by one",
    "",
);
pub static PYTHON_TERNARY: Habit = shape(
    "python_ternary",
    codes::REMOVED_SYNTAX,
    "an inline `if` takes `:` after the condition and after `else`",
    "",
);
pub static REVERSED_SLICE: Habit = shape(
    "reversed_slice",
    codes::EXPECTED_TOKEN,
    "`xs.rev` is the reversed view; `List(xs.rev)` copies it",
    "",
);
pub static REVERSED_SLICE_2: Habit = shape(
    "reversed_slice_2",
    codes::EXPECTED_TOKEN,
    "write `.rev`",
    "",
);
pub static REVERSE_VIEW_HELP: Habit = shape(
    "reverse_view_help",
    codes::LITERAL_SLICE_INDEX,
    "{}, or {}",
    "",
);
pub static RUST_CLOSURE: Habit = shape(
    "rust_closure",
    codes::EXPECTED_TOKEN,
    "write `x => x * 2`, or `(a, b) => a + b`",
    "",
);
pub static RUST_PRINT: Habit = shape(
    "rust_print",
    codes::EXPECTED_TOKEN,
    "`print(a, b)` writes its values with spaces between, then a newline; `\"{{x}}\"` interpolates",
    "",
);
pub static RUST_REPEAT: Habit = shape(
    "rust_repeat",
    codes::REMOVED_SYNTAX,
    "write `[n].filled(x)`, a list of `n` copies of `x`",
    "",
);
pub static STATEMENT_END_HELP: Habit = shape(
    "statement_end_help",
    codes::UNEXPECTED_TOKEN,
    "statements end at the end of the line; `;` is only used inside `{{ ... }}`",
    "",
);
pub static STATEMENT_END_HELP_2: Habit = shape(
    "statement_end_help_2",
    codes::UNEXPECTED_TOKEN,
    "remove `;`",
    "",
);
pub static STRUCT_LITERAL: Habit =
    shape("struct_literal", codes::UNKNOWN_FIELD, "write `{}({})`", "");
pub static SUM: Habit = shape(
    "sum",
    codes::PARENS_MEAN_WORK,
    "write `{}()`: `()` marks a call that does work",
    "",
);
pub static SUM_2: Habit = shape("sum_2", codes::PARENS_MEAN_WORK, "write `{}()`", "");
pub static C_ADDRESS_2: Habit = shape(
    "c_address_2",
    codes::MISSING_MUT_AT_CALL,
    "drop the `&`",
    "",
);
pub static UNDECLARED_RETURN: Habit = shape(
    "undeclared_return",
    codes::TYPE_MISMATCH,
    "a fn without `-> T` gives nothing: add `-> {}`",
    "",
);
pub static UNKNOWN_MEMBER: Habit = shape(
    "unknown_member",
    codes::UNKNOWN_FIELD,
    "a library domain's `{}` is `{}` since 0.3",
    "",
);
pub static UNKNOWN_MEMBER_10: Habit = shape(
    "unknown_member_10",
    codes::UNKNOWN_FIELD,
    "its methods are {}",
    "the methods listed are choices, not one edit",
);
pub static UNKNOWN_MEMBER_2: Habit = shape("unknown_member_2", codes::UNKNOWN_FIELD, "{} is a library domain (`impl Index`): it walks in `ord` order, but neighbours, cuts, `at` and `wrap` belong to the ordered core domains; use `x.ord` and `{}.nth(k)`", "a library domain has no neighbours, so the walk is rewritten around `ord`");
pub static UNKNOWN_MEMBER_3: Habit = shape(
    "unknown_member_3",
    codes::UNKNOWN_FIELD,
    "`{}` is spelled `{}` in CP Lang",
    "",
);
pub static UNKNOWN_MEMBER_4: Habit = shape(
    "unknown_member_4",
    codes::UNKNOWN_FIELD,
    "a {} is empty when its `len` is 0",
    "",
);
pub static UNKNOWN_MEMBER_5: Habit = shape(
    "unknown_member_5",
    codes::UNKNOWN_FIELD,
    "write `len == 0`",
    "",
);
pub static UNKNOWN_MEMBER_6: Habit = shape(
    "unknown_member_6",
    codes::UNKNOWN_FIELD,
    "`{}` shows its top with `peek`, `None` when empty",
    "",
);
pub static UNKNOWN_MEMBER_7: Habit = shape(
    "unknown_member_7",
    codes::UNKNOWN_FIELD,
    "a {} combines a range with `fold`",
    "",
);
pub static UNKNOWN_MEMBER_8: Habit =
    shape("unknown_member_8", codes::UNKNOWN_FIELD, "write `fold`", "");
pub static UNKNOWN_MEMBER_9: Habit = shape(
    "unknown_member_9",
    codes::UNKNOWN_FIELD,
    "its fields are {}",
    "the fields listed are choices, not one edit",
);
pub static WITHOUT: Habit = shape("without", codes::TYPE_MISMATCH, "add `-> {}`", "");
pub static LITERAL: Habit = shape("literal", codes::VALUE_NAME_CASE, "make it a `const`", "");
pub static INDEX_OF: Habit = shape(
    "index_of",
    codes::UNKNOWN_FIELD,
    "a value's first position is `find`, `None` when it isn't there: `{}`",
    "",
);
pub static PARTITION_POINT: Habit = shape("partition_point", codes::UNKNOWN_FIELD, "`partition_point` is where its test turns false; `bsearch` takes the test that turns true there: `{}`", "");
pub static STR_TEST: Habit = shape(
    "str_test",
    codes::UNKNOWN_FIELD,
    "`{}` tests a `Char`; a string tests every character with `all`: `{}`",
    "",
);
pub static BY_TWIN: Habit = shape("by_twin", codes::UNRESOLVED_NAME, "write `{}(by: ..)`", "");
pub static RENAME_VALUES: Habit = shape(
    "rename_values",
    codes::VALUE_NAME_CASE,
    "rename `{}` to `{}` everywhere",
    "",
);
pub static SAME_PLACE: Habit = shape(
    "same_place",
    codes::SAME_PLACE_READ_WRITE,
    "compute the right side {}: otherwise the order the two sides run in decides the result",
    "",
);
pub static TUPLE_ASSIGN: Habit = shape(
    "tuple_assign",
    codes::UNEXPECTED_TOKEN,
    "CP Lang reads numbers by token, so the names are declared as a tuple read",
    "",
);
pub static TUPLE_ASSIGN_2: Habit = shape(
    "tuple_assign_2",
    codes::UNEXPECTED_TOKEN,
    "several names are assigned at once in parentheses: `(a, b) = (b, a)`",
    "",
);
pub static RUST_REPEAT_2: Habit = shape(
    "rust_repeat_2",
    codes::REMOVED_SYNTAX,
    "write `[{}].filled({})`",
    "",
);
pub static C_CAST: Habit = shape("c_cast", codes::REMOVED_SYNTAX, "write `{}(...)`", "");
pub static STRUCT_LITERAL_2: Habit = shape(
    "struct_literal_2",
    codes::EXPECTED_TOKEN,
    "use parentheses",
    "",
);
pub static PYTHON_FORMAT: Habit = shape(
    "python_format",
    codes::EXPECTED_TOKEN,
    "write each value in its hole: `\"{{x}} {{y:.2}}\"`",
    "the holes and the values don't pair up, so which value goes where is the writer's call",
);
pub static BARE_TUPLE_PATTERN_2: Habit = shape(
    "bare_tuple_pattern_2",
    codes::EXPECTED_TOKEN,
    "add the parentheses",
    "",
);
pub static PAREN_FOR_2: Habit = shape(
    "paren_for_2",
    codes::REMOVED_SYNTAX,
    "remove the parentheses",
    "",
);
pub static EMPTY_DICT: Habit = shape(
    "empty_dict",
    codes::TYPE_MISMATCH,
    "an empty {} is `{}`; its key and value types come from how it is used",
    "",
);
pub static ITERATOR_INDEX: Habit = shape(
    "iterator_index",
    codes::UNKNOWN_FIELD,
    "write the index",
    "",
);
pub static HEAP_METHODS: Habit = shape(
    "heap_methods",
    codes::UNRESOLVED_NAME,
    "a heap is std's `Heap`, with methods: `{}`",
    "",
);
pub static HEAP: Habit = shape("heap", codes::UNRESOLVED_NAME, "a heap is std's `Heap`, not a list: `var h: Heap<Int> = Heap()` (`use std.heap.{{Heap}}`), then `h.push(x)` and `h.pop()!`", "a list becomes a `Heap` where it is declared, another line than the call");
pub static C_CAST_FLOAT: Habit = shape(
    "c_cast_float",
    codes::REMOVED_SYNTAX,
    "a float is made with `F64(x)`",
    "",
);
pub static C_CAST_INT: Habit = shape(
    "c_cast_int",
    codes::REMOVED_SYNTAX,
    "an integer is made with `Int(x)`, and an `Int` is already 64 bits",
    "",
);

/// Every shape row; `all` walks them after the named rows.
pub static SHAPES: &[&Habit] = &[
    &ARROW_LAMBDA,
    &ARROW_LAMBDA_2,
    &WRITE,
    &BAD_OPERANDS_2,
    &BAD_OPERANDS_3,
    &BARE_TUPLE_PATTERN,
    &CHANGES,
    &CHAR_LITERAL_FIX,
    &CMP_TOKEN,
    &CMP_TOKEN_2,
    &CMP_TOKEN_3,
    &COLON_RETURN_TYPE,
    &COLON_RETURN_TYPE_2,
    &CPP_NPOS,
    &CPP_NPOS_2,
    &C_ADDRESS,
    &C_AUTO,
    &C_AUTO_2,
    &C_CASE,
    &C_CASE_2,
    &C_COUNTDOWN_REPORT,
    &C_COUNTDOWN_REPORT_2,
    &C_DECL_ERROR,
    &C_DEREF,
    &C_FOR,
    &C_FOR_2,
    &C_RANGE_FOR,
    &C_RANGE_FOR_2,
    &C_RANGE_FOR_3,
    &C_TERNARY,
    &C_TERNARY_2,
    &DUPLICATE_TAG,
    &EMPTY_BRACES,
    &EMPTY_TYPE_ARGS,
    &EMPTY_TYPE_ARGS_2,
    &ENUM_TAGS,
    &FOREIGN_NEW,
    &FOREIGN_NEW_2,
    &FOREIGN_STEP,
    &FOREIGN_STEP_2,
    &FOREIGN_THEN,
    &FOREIGN_THEN_2,
    &FOR_OF,
    &F_STRING_PREFIX,
    &F_STRING_PREFIX_2,
    &GETS,
    &INDEX_ITEM,
    &INDEX_ITEM_2,
    &ITER_CALL_HELP,
    &ITER_CALL_HELP_2,
    &KEYED_FIND,
    &KEYWORD_ARG,
    &LAZY_LEN,
    &LAZY_LEN_2,
    &LIST_IN,
    &LIST_IN_2,
    &LIST_PATTERN,
    &LOOP_ADVICE,
    &MEMBER_ADVICE,
    &MEMBER_ADVICE_2,
    &MEMBER_ADVICE_3,
    &MEMBER_ADVICE_4,
    &MEMBER_FOR_CALL,
    &MEMBER_FOR_CALL_2,
    &MEMBER_FOR_CALL_3,
    &NO_OPERATOR_IMPL,
    &NO_OPERATOR_IMPL_2,
    &NO_OPERATOR_IMPL_3,
    &OLD_OPTIONAL_SPELLING,
    &PAREN_FOR,
    &PYTHON_SPLAT,
    &PYTHON_SPLAT_2,
    &PYTHON_TERNARY,
    &REVERSED_SLICE,
    &REVERSED_SLICE_2,
    &REVERSE_VIEW_HELP,
    &RUST_CLOSURE,
    &RUST_PRINT,
    &RUST_REPEAT,
    &STATEMENT_END_HELP,
    &STATEMENT_END_HELP_2,
    &STRUCT_LITERAL,
    &SUM,
    &SUM_2,
    &C_ADDRESS_2,
    &UNDECLARED_RETURN,
    &UNKNOWN_MEMBER,
    &UNKNOWN_MEMBER_10,
    &UNKNOWN_MEMBER_2,
    &UNKNOWN_MEMBER_3,
    &UNKNOWN_MEMBER_4,
    &UNKNOWN_MEMBER_5,
    &UNKNOWN_MEMBER_6,
    &UNKNOWN_MEMBER_7,
    &UNKNOWN_MEMBER_8,
    &UNKNOWN_MEMBER_9,
    &WITHOUT,
    &LITERAL,
    &INDEX_OF,
    &PARTITION_POINT,
    &STR_TEST,
    &BY_TWIN,
    &RENAME_VALUES,
    &SAME_PLACE,
    &TUPLE_ASSIGN,
    &TUPLE_ASSIGN_2,
    &RUST_REPEAT_2,
    &C_CAST,
    &STRUCT_LITERAL_2,
    &PYTHON_FORMAT,
    &BARE_TUPLE_PATTERN_2,
    &PAREN_FOR_2,
    &EMPTY_DICT,
    &ITERATOR_INDEX,
    &HEAP_METHODS,
    &HEAP,
    &C_CAST_FLOAT,
    &C_CAST_INT,
];
