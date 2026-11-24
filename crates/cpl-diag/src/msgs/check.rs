//! Messages check emits, and the lowering, build and editor messages
//! about a checked program.

super::msgs! {
    A_SECOND_DIRECTIVE = UNEXPECTED_TOKEN, "a second `@!{}`";
    WITH_NOT_FIRST = WITH_NOT_FIRST, "a bare `with` line comes first in its block; indent what it covers under `with ...:`";
    FN_OVERLOAD = OVERLOAD_BY_TYPE, "a second `fn {}`: fns are not overloaded by their parameter types";
    DEFINED_TWICE = DUPLICATE_DEFINITION, "`{}` is already defined in this file";
    IMPORTED_TWICE = DUPLICATE_DEFINITION, "`{}` is already imported by `use` in this file";
    WIDE_ARRAY = WIDE_ARRAY, "`{}` stores 8-byte `Int`s, but every value written into it is in {}..={}";
    PADDING = PADDING, "each element holds {} bytes of data in {}, so {} of this {} MB array is padding";
    LET_NOT_HOISTED = UNRESOLVED_NAME, "`{}` is not visible here: a `let` is visible only from line {} on, and lambdas are not hoisted";
    /// Its code is the walk's kind: a lazy chain or a generator.
    WALKED_TWICE = CHAIN_WALKED_TWICE, "`{}` is {}, and this walks it a second time";
    USED_IN_OWN_DEFINITION = USED_BEFORE_SET, "`{}` is used in its own definition";
    /// `UNRESOLVED_NAME` instead when the use is in a lambda.
    USED_BEFORE_SET = USED_BEFORE_SET, "`{}` is used here before it is set on line {}";
    PRIVATE_NAME = PRIVATE_NAME, "`{}` is private to `{}`";
    MODULE_HAS_NO = UNRESOLVED_NAME, "`{}` has no `{}`";
    MODULE_HAS_NO_REMOVED = UNRESOLVED_NAME, "`{}` has no `{}`: std removed it";
    MODULE_HAS_NO_NOT_STD = UNRESOLVED_NAME, "`{}` has no `{}`, and not in the stdlib (yet)";
    IMPORT_EACH_OTHER = UNRESOLVED_MODULE, "`{}` imports this file, so they import each other";
    IMPORT_BACK = UNRESOLVED_MODULE, "`{}` imports this file back: {}";
    IN_IMPORT_CYCLE = UNRESOLVED_MODULE, "`{}` is in an import cycle: {}";
    STD_NOT_LOADED = UNRESOLVED_MODULE, "std module `{}` is not loaded yet";
    NO_MODULE_NOT_STD = UNRESOLVED_MODULE, "no module `{}`, and not in the stdlib (yet)";
    NO_MODULE = UNRESOLVED_MODULE, "no module `{}`";
    RESULT_DROPPED = UNUSED_VALUE, "`{}` returns {}, and it is dropped";
    CACHE_FIELD_WRITTEN = CACHE_FIELD_WRITTEN_OUTSIDE, "`{}.{}` is a `@cache` field and should only be written inside `{}`";
    MUT_PARAM_UNUSED = MUT_PARAM_UNUSED, "`{}` is taken `mut`, but the fn never changes it";
    MEMORY_OVER_PROFILE = MEMORY_OVER_PROFILE, "at the input bounds this takes {}, over the {} MB limit on {}: {}";
    FRESH_TYPE_LEAKS = FRESH_TYPE_LEAKS_TEST, "`{}` is declared above `@!multitest`, so a `{}` stored in it would outlive its test case";
    PARENS_MEAN_WORK = PARENS_MEAN_WORK, "`{}` does work, but without `()` it reads like an O(1) field";
    SAME_PLACE = SAME_PLACE_READ_WRITE, "`{}` is changed on the right side of an assignment to `{}`, and {}";
    /// The `{}` reason of `SAME_PLACE` when one place holds the other.
    PART_OF = SAME_PLACE_READ_WRITE, "`{}` is part of `{}`";
    SLICE_KEY_READ = LITERAL_SLICE_INDEX, "`{}[{}]` reads source key {}: a slice keeps its source's keys, so this is not its element {}";
    SLICE_COUNTS_FROM_ZERO = LITERAL_SLICE_INDEX, "`0..{}.len` counts from 0, but the slice `{}` keeps its source's keys";
    VIEW_REREAD_IN_LOOP = MAP_RECOMPUTED, "`{}` is a lazy `.{}` view and a loop reads it again on every pass: each read reruns {}";
    VIEW_REREAD_BY_INDEX = MAP_RECOMPUTED, "`{}` is a lazy `.{}` view and it is read {} times by index: each read reruns {}";
    LENT_CLOSURES_KEPT = LENT_ESCAPES, "`{}` yields closures that write its local `{}`, and this use keeps them after `{}` returns; loop over it with a plain `for` instead";
    NOT_A_DOMAIN_IN_LIBRARY = TYPE_MISMATCH, "`{}` (`{}`) needs a domain for `{}`, and {} is not one";
    NOT_A_DOMAIN = TYPE_MISMATCH, "`{}` indexes an array here, so it must be a domain, and {} is not one";
    PARAM_TYPES_CYCLE = DEFINITION_CYCLE, "`{}`'s parameter types depend on `{}` itself";
    VALUE_DROPPED = UNUSED_VALUE, "this `{}` is worked out and then dropped";
    STR_METHOD_ON_CHARS = TYPE_MISMATCH, "this method is called on a `Str`, and `{}` is a `List<Char>`";
    PARSE_NEEDS_TYPE = WRONG_ARG_COUNT, "`parse` needs the type it reads: `{}.parse<{}>()`";
    IF_BRANCHES_DIFFER = TYPE_MISMATCH, "the branches of this `if` have different types: `{}` and `{}`";
    INTERNAL_ERROR = INTERNAL, "internal error in cpl while handling `{}`: {}";
    OVERFLOWS_WHEN = OVERFLOW, "`{}` overflows Int when {}";
    SIZE_OVER_TARGET = SIZE_NEAR_LIMIT, "the C++ for `{}` is {} bytes, over the {}-byte target and {}% of the {}-byte limit of judge `{}`; `--size` says where the bytes go";
    SIZE_NEAR_TARGET = SIZE_NEAR_LIMIT, "the C++ for `{}` is {} bytes, over {} bytes, near the {}-byte target and {}% of the {}-byte limit of judge `{}`; `--size` says where the bytes go";
    SIZE_AS_WRITTEN = SIZE_NEAR_LIMIT, "as written, the C++ for `{}` would be {} bytes, over the {} of judge `{}`; {}";
}
