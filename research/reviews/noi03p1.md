# noi03p1

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`, a fragment). The weighted-digit idea is right: a total of `weight * digit` that must be 0, single reshapes, then one-lose/one-gain pairs, first fix printed (lines 42-69).

## Checklist failures

- idiomatic-code/R3: `eq[at].ord - '0'.ord` (line 40) where std `Char.digit` exists, and `Char('0'.ord + d)` (line 66) by character arithmetic.
- idiomatic-code/R4: `loses` and `gains` (lines 19-20) are the same comprehension with `<` and `>`.
- The `moves` comprehension (lines 46-63) is formatted one argument per line by fmt, so the condition `total + gain(a, x) + gain(b, y) == 0` spans eleven lines.
- The parse (lines 27-41) is a hand `while` with a cursor; reported as "no tokenizer for signed numbers" without a run.

## Owed rewrites

- Name the test as the dream does (`fixed(a, d)`, `fixed(a, x, b, y)`) so each comprehension fits on a line; `eq[at].digit`.
- Digit values as `0..=9` in `Digit`, `gain` and the tables (lines 16-22).
- Either quote a `cpl run` of a signed-number split for the tokenizer claim, or rewrite the parse with `split`/`split_once` on the operators.

## Language gaps

- Const domain table for stick counts: in the friction.

## Dream notes

`for d in Shape`, `reshape(a, d)`, `fixed(...)` and `eq.with(m)` read as the editorial; the fragment skips the parse, which is where the port loses most.
