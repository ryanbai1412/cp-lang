//! Messages cpl-interp traps with: a run's failure, as `cpl run`, trace and
//! explain show it.

super::msgs! {
    ARRAY_CAN_HAVE_LENGTH = RUNTIME, "an array can't have length {}";
    CHAR_INT_LEFT_ASCII_0 = RUNTIME, "`Char + Int` left ASCII (0..=127)";
    CHECKPOINT_AT_NO_LOOP_AND = RUNTIME, "the checkpoint is at no loop and no call";
    CHECKPOINT_DOES_NOT_MATCH_PROGRAM = RUNTIME, "the checkpoint does not match the program";
    CMP_MUST_RETURN_ORDERING = RUNTIME, "`cmp` must return an `Ordering`";
    CUTS_NEED_SEQUENCE = RUNTIME, "cuts need a sequence";
    CYC_ON_EMPTY_DOMAIN = RUNTIME, "cyc on empty domain";
    DIV_BY_ZERO_OP = DIVISION_BY_ZERO, "division by zero: `{} {} 0`";
    DOES_NOT_FIT_IN = OVERFLOW, "`{}` does not fit in {}";
    DOES_NOT_FIT_IN_INT = OVERFLOW, "`{}` does not fit in an `Int`";
    DOES_NOT_FIT_IN_INT_2 = OVERFLOW, "{} does not fit in an `Int`";
    DOES_NOT_FIT_IN_U64 = OVERFLOW, "{} does not fit in a `U64`";
    EMPTY = RUNTIME, "the {} is empty";
    ENDED_WHERE_CHARACTER_WAS_EXPECTED = RUNTIME, "the {} ended where a character was expected";
    /// An interpreter bug: a value of a shape lowering should have ruled out.
    /// The argument names the site, so a report still pins the spot.
    INTERPRETER_BUG = RUNTIME, "internal: {}";
    INPUT_ENDED_FOR_CHARACTER = BAD_INPUT_TOKEN, "the input ended where a character was expected";
    INPUT_ENDED_WHERE_WAS_EXPECTED = BAD_INPUT_TOKEN, "the input ended where {} was expected";
    ENDED_WHERE_WAS_EXPECTED = RUNTIME, "the {} ended where {} was expected";
    EXPECTED_NUMBER_FOUND = BAD_INPUT_TOKEN, "expected a number, found `{}`";
    HAS_NO_POSITION_IT = RUNTIME, "`{}` has no position {}: it is {}";
    INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH = INDEX_OUT_OF_RANGE, "index {} is outside an array dimension of length {}";
    INDEX_OUTSIDE_ARRAY_LENGTH = INDEX_OUT_OF_RANGE, "index {} is outside an array of length {}";
    INDEX_OUTSIDE_ARRAY_SHAPE = INDEX_OUT_OF_RANGE, "index ({}, {}) is outside an array of shape ({}, {})";
    INDEX_OUTSIDE_AXIS_LENGTH = INDEX_OUT_OF_RANGE, "index {} is outside an axis of length {}";
    INDEX_OUTSIDE_LIST_LENGTH = INDEX_OUT_OF_RANGE, "index {} is outside a list of length {}";
    INDEX_OUTSIDE_RANGE_LENGTH = INDEX_OUT_OF_RANGE, "index {} is outside a range of length {}";
    INDEX_OUTSIDE_STRING_LENGTH = INDEX_OUT_OF_RANGE, "index {} is outside a string of length {}";
    INSERT_OUTSIDE_LIST_LENGTH = RUNTIME, "insert({}) is outside a list of length {}";
    INTEGER_OVERFLOW = OVERFLOW, "integer overflow: {}";
    INTEGER_OVERFLOW_ABS_DOES_NOT = OVERFLOW, "integer overflow: `abs({})` does not fit in an `Int` (64 bits)";
    INTEGER_OVERFLOW_ABS_I128 = OVERFLOW, "integer overflow: `abs({})` does not fit in an `I128` (128 bits)";
    INTEGER_OVERFLOW_BIT_CEIL_DOES = OVERFLOW, "integer overflow: `bit_ceil({})` does not fit in an `Int` (64 bits)";
    INTEGER_OVERFLOW_DOES_NOT_FIT = OVERFLOW, "integer overflow: `-{}` does not fit in an `Int` (64 bits)";
    INTEGER_OVERFLOW_DOES_NOT_FIT_2 = OVERFLOW, "integer overflow: `-{}` does not fit in an `I128` (128 bits)";
    INT_OVERFLOW_BINOP = OVERFLOW, "integer overflow: `{} {} {}` does not fit in an `Int` (64 bits)";
    I128_OVERFLOW_BINOP = OVERFLOW, "integer overflow: `{} {} {}` does not fit in an `I128` (128 bits)";
    INTEGER_OVERFLOW_RANGE_LENGTH_DOES = OVERFLOW, "integer overflow: range `{}..{}` length does not fit in an `Int` (64 bits)";
    KEYS_NEED_SLICE = RUNTIME, "keys need a slice";
    KEY_NOT_IN_MAP_M = RUNTIME, "the key `{}` is not in the map; `m.get(k)` gives an optional";
    KEY_NOT_ONE_VIEW_KEYS = INDEX_OUT_OF_RANGE, "key {} is not one of the view's keys {}..{}";
    KEY_NOT_ONE_VIEW_KEYS_2 = INDEX_OUT_OF_RANGE, "key {} is not one of the view's keys {}..{}: `.nth(k)` reads by position";
    LENT = RUNTIME, "this variable is being changed by a call that has not returned yet";
    LIST_FILLED_WITH_NEGATIVE_LENGTH = RUNTIME, "List.filled with negative length {}";
    LN_NEGATIVE_NUMBER = RUNTIME, "ln({}) of a negative number";
    NEEDS_POWER_THAT_NOT_NEGATIVE = RUNTIME, "`**` needs a power that is not negative, and this is {}";
    NEGATIVE_NUMBER = RUNTIME, "`<<` of the negative number {}";
    NOT_BOOL_WHICH_READ_AS = RUNTIME, "{} is not a `Bool`, which is read as 0 or 1";
    NOT_INT = RUNTIME, "`{}` is not an Int";
    NOT_IN_WHICH = DOMAIN_CAST, "{} is not in `{}`, which is {}";
    ON_NONE_THERE_NO_VALUE = UNWRAP_NONE, "`!` on `None`: there is no value here";
    OUTSIDE = RANGE_STORE, "{} is outside `{}`";
    OUTSIDE_0_127 = RANGE_STORE, "{} is outside `0..=127`";
    OUTSIDE_1_1 = RUNTIME, "{}({}) is outside -1..=1";
    OVERFLOWS_INT_BINOP = RUNTIME, "`{} {} {}` overflows `Int`";
    POP_FROM_EMPTY_LIST = RUNTIME, "pop from an empty list";
    QUERY_LIMIT_EXCEEDED_THIS_QUERY = RUNTIME, "query limit exceeded: this is query {}, and the limit is {}";
    RAND_LO_HI = RUNTIME, "rand({}, {}): lo > hi";
    RANGE_END_BEFORE_START = RUNTIME, "range end {} is before start {}";
    RANGE_STEPS_FORWARD_BY_NOT = RUNTIME, "a range steps forward: `by {}` is not positive";
    REACH_WHILE_PREDICATE_NOT_MONOTONE = RUNTIME, "reach: `while` predicate is not monotone (reach dropped by more than one)";
    READING_NUMBER_TESTS = RUNTIME, "reading the number of tests: {}";
    REKEY_NEEDS_AS_MANY_KEYS = RUNTIME, "rekey needs as many keys as cells: {}..{} for {} cells";
    REKEY_NEEDS_LIST = RUNTIME, "rekey needs a list";
    REMOVE_OUTSIDE_LIST_LENGTH = RUNTIME, "remove({}) is outside a list of length {}";
    RESERVE_TO_NEGATIVE_CAPACITY = RUNTIME, "reserve to negative capacity {}";
    RESIZE_TO_NEGATIVE_LENGTH = RUNTIME, "resize to negative length {}";
    SHIFT_BY_OUTSIDE_0_128 = RUNTIME, "a shift by {} is outside 0..128";
    SHIFT_BY_OUTSIDE_0_64 = RUNTIME, "a shift by {} is outside 0..64";
    SLICE_EXPECTS_LIST = RUNTIME, "slice expects a list";
    SLICE_EXPECTS_LIST_OR_ARRAY = RUNTIME, "slice expects a list or array";
    SLICE_EXPECTS_STRING = RUNTIME, "slice expects a string";
    SLICE_OUTSIDE_KEYS = INDEX_OUT_OF_RANGE, "slice {}..{} is outside the keys {}..{}";
    SLICE_OUTSIDE_LIST_LENGTH = INDEX_OUT_OF_RANGE, "slice {}..{} is outside a list of length {}";
    SLICE_OUTSIDE_STRING_LENGTH = INDEX_OUT_OF_RANGE, "slice {}..{} is outside a string of length {}";
    SORT_CMP_COMPARATOR_NOT_STRICT = RUNTIME, "sort `cmp:` comparator is not a strict weak order: cmp(x, x) is true";
    SORT_CMP_COMPARATOR_NOT_STRICT_2 = RUNTIME, "sort `cmp:` comparator is not a strict weak order: result is not ordered";
    SORT_CMP_COMPARATOR_NOT_STRICT_3 = RUNTIME, "sort `cmp:` comparator is not a strict weak order: both cmp(a, b) and cmp(b, a) are true";
    SQRT_NEGATIVE_NUMBER = RUNTIME, "sqrt({}) of a negative number";
    STACK_RAN_OUT_NEVER_STOPS = STACK_EXHAUSTED, "the stack ran out: {} never stops calling itself?";
    STOPPED = RUNTIME, "the call to replay returned";
    SWAP_OUTSIDE_ARRAY_LENGTH = INDEX_OUT_OF_RANGE, "swap({}, {}) is outside an array of length {}";
    SWAP_OUTSIDE_LIST_LENGTH = INDEX_OUT_OF_RANGE, "swap({}, {}) is outside a list of length {}";
    THERE_NO_SOLUTION_TO_TALK = RUNTIME, "there is no solution to talk to: an `@interactor` runs under `cpl run` or `cpl stress`";
    THERE_NO_VALUE_HERE_IT = UNWRAP_NONE, "there is no value here: it is `None`";
    THIS_ARRAY_TOO_LARGE_TO = RUNTIME, "this array is too large to allocate";
    THIS_KEY_NOT_IN_MAP = RUNTIME, "this key is not in the map; store with `m[k] = v` first";
    TRUNCATE_TO_NEGATIVE_LENGTH = RUNTIME, "truncate to negative length {}";
    UNWRAPPED_NONE_VALUE = UNWRAP_NONE, "unwrapped a `None` value";
    WINDOW_LEAVES_DOMAIN_0 = RUNTIME, "window of {} leaves domain (0..={})";
    CPP_ONLY = RUNTIME, "this fn's body is C++ (`@cpp`), so only the C++ build runs it: `cpl run` does that by itself";
    NO_GRADER = RUNTIME, "this fn has no body: the judge's grader supplies it, so there is nothing to run here";
    OUT_OF_TIME = RUNTIME, "the run took longer than its time limit";
    CANCELLED = RUNTIME, "the run was stopped";
    INTERPRETER_FAILED = RUNTIME, "the interpreter itself failed; this is a bug in cpl";
    NO_INTERPRETER_THREAD = RUNTIME, "could not start the interpreter thread";
    /// The first of `a op b`, the operator, then the second.
    /// What was expected, then the word read and its length.
    WORD_WRONG_LENGTH = BAD_INPUT_TOKEN, "expected a `{}`, found `{}`, of length {}";
    /// What was expected, the word read, then the character not in its alphabet.
    WORD_OUTSIDE_ALPHABET = BAD_INPUT_TOKEN, "expected a `{}`, found `{}`: `{}` is not in its alphabet";
    /// What failed to read, then where in the input.
    READ_FAILED_AT = RUNTIME, "{} ({})";
    /// Text the program or its input wrote: an `assert`'s message, a lowered
    /// `Trap`, a read's failure.
    WRITTEN = RUNTIME, "{}";
}
