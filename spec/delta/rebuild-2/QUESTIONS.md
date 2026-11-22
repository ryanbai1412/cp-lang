# Rebuild round 2: forty snippet questions

Each question is a snippet. The answer says whether it compiles. If it does,
the answer gives the output. If it does not, the answer gives the error and
the rule behind it. Codes come from `spec/diagnostics.md`. An answer marked
(guess) points to a hole in `HOLES.md`.

## Lexical grammar

Q1. `let x = 5.`
No: E0005 bad_number. A float needs a fraction, and `5.` is an error.
(R01.4.2)

Q2. `print(-7 / 2, -7 % 2)`
Yes, it prints `-4 1`. `/` is floor division and `%` is never negative.
(R01.5.2)

Q3. `let N = 5`
No: E0208 value_name_case. Values are lowercase, so `N` would read as a type.
(R01.3.2)

Q4. `print(1e9 + 7)`
Yes, it prints `1000000007`. A scientific literal that is an exact integer
is an `Int`. (R01.4.1)

Q5. `let c = 'ab'`
No: an error that suggests `"ab"`. (R01.4.3) The code is probably E0007
bad_char_literal, but the catalogue's example for E0007 is a non-ASCII
char (guess, H-DIAG-CODES).

Q6. `print(-2 ** 2)`
Yes, it prints `-4`. `**` binds tighter than unary minus. (R01.5.1)

Q7. `let a = 1` then `print(yes(0 <= a < 2))`
Yes, it prints `YES`. The chain points one way, and `yes` gives the
answer word. (R01.5.3, R10.3.3)

Q8. `print(yes(1 < 2 == true))`
No: E0111 mixed_comparison_chain. A chain that contains `==` is an
error. (R01.5.3)

Q9. `let s = 6` then `print(yes(s & (s << 1) == 4))`
Yes, it prints `YES`. `&` binds tighter than `==`, so this is
`(6 & 12) == 4`. (R01.5.1)

Q10. `let x = 1 # note`
No: E0004 unknown_char. `#` is not a comment, and the fix is `//`. (R01.1.2,
R01.11.1)

Q11. `let b = [1] ++ [2]`
No: a lex error. CP Lang concatenates with `+`. (R01.4.6) The conformance
case expects E0004. The catalogue describes E0004 only as "`#` is not a
comment" (H-DIAG-CODES).

## Types and optionals

Q12. `let d: ?Int = None` then `if d + 1 < 5: print(1)`
No: E0308 optional_compare. Arithmetic lifts, but ordering does not.
(R02.4.4, R02.4.5)

Q13. `var d: ?Int = None` then `d min= 5`, `d min= 7`, `print(d ?? 0)`
Yes, it prints `5`. A `None` target takes the right side. (R02.4.6)

Q14. `var x = None` then `x = 3`
No: E0806 inference_needs_annotation. Inference never looks at later
statements. (R02.10.1)

Q15. `let a = "ab"`, `let b = a`, `print(a)`
No: E0608 heap_copy. A `Str` is a heap value, so `let b = a` moves it.
(R02.1.2, R05 R2)

Q16. `struct E: u: Int, w: Int` then `var e = E(1, 2)` and `e.w = 3`
No: E0405 assign_fixed_field. Only `var` fields can change after
construction. (R02.6.1)

Q17. `let f: F64 = 3`
No: E0302 type_mismatch. `Int` never converts to `F64` implicitly; write
`F64(3)`. (R02.11.1, R04.4.22)

Q18. `let d: ?Int = None` then `if d? > 0: print(1)`
No: E0803 question_in_condition. (R02.4.7)

## Numbers, overflow and niches

Q19. `var x = 4e18` then `x = x * 3`
It compiles. A checked build traps with R0202 overflow. A release build
wraps. (R03.4.1, R09.3.4)

Q20. `var a: [3]0..=9 = [3].filled(0)` then `a[0] = 10`
No: E0902 range_store_unprovable_const. The constant is outside the range
type. (R03.1.4)

Q21. `type Z = Mod<7>` then `print(Z(5) + Z(3))`
Yes, it prints `1`. `Mod` is closed under `+`. (R03.5.1) Nothing states how
`Mod` prints (H-MOD-PRINT).

Q22. `domain V = 0..read<1..=2e5>()` then `var par: [V]?V = None`. What
does `cpl explain` report for `par`?
It reports 4 bytes per element, with `None` as the all-ones pattern. Positions
take the narrowest width that holds `V.count + 1` values, and `?V` uses the
spare value. (R03.1.1, R03.2.1, R13.2.5)

Q23. `read<Int>()` on the input `-9223372036854775808`
It compiles. A checked run rejects the token, because it equals the spare
value of `Int`: R0502, input out of its read contract (guess between R0501
and R0502). (R03.3.4, R10.2.6)

## Domains, positions and cuts

Q24. `domain V = 1..=4` then `let w: V = V(2) + 1`
No: E0302. Arithmetic on a position gives an `Int`. (R04.8.2)

Q25. `domain V = 1..=3` then `let v: V = V.first`
No: E0302. `V.first` is a `?V`. (R04.2.3)

Q26. `domain V = 1..=4`, `let a = [V] v => Int(v) * 2`, `print(a[3])`
Yes, it prints `6`. The literal `3` is the position 3. (R04.2.18)

Q27. `domain V = 1..=5` then `print(Int(V.cuts.last!))`
Yes, it prints `5`. `Int(c)` counts the positions before the cut, whatever
the base. (R04.2.5, R04.2.16)

Q28. `domain V = 1..=5`, `let a = [V] v => Int(v)`, `print(a[..3].sum())`
Yes, it prints `3`. A literal end is a position, so `..3` is the positions
1 and 2. (R04.4.8, R07.9.1)

Q29. `fn cost(r: Range<V>) -> Int = 0`, `let s = V.cuts(1)..V.cuts(3)`,
`print(cost(s))`
No: E0302. Only a range literal with cut ends becomes a piece; a stored cut
range does not. (R04.4.5)

Q30. `print((1..=10).first_true(x => x * x >= 10)!)`
Yes, it prints `4`. An integer range is an anonymous domain. (R04.4.19,
R04.4.28)

## Ownership and lending

Q31. `fn grow(mut xs: List<Int>): xs.push(1)`, `var a = List<Int>()`,
`grow(a)`
No: E0402 missing_mut_at_call. The fix is `grow(mut a)`. (R06.1.2, R05 R4)

Q32. `fn same(xs: List<Int>) -> List<Int> = xs`
No: E0603 lent_escapes. The fn returns a lent parameter without declaring
`lends`. (R05.7.8)

Q33. `var xs = [1, 2, 3]`, `let w = xs[0..2]`, `xs.push(4)`, `print(w.len)`
No: E0602 used_while_lent. A change of length while a view is alive is a
compile error. (R07.5.9)

Q34. `var f = (n: Int) -> Int => if n == 0: 0 else: f(n - 1)`
No: E0406 var_lambda_recursion. Only a `let`-bound lambda may call itself.
(R06.2.3)

## Sequences

Q35. `let a = [1, 2, 3, 4, 5]` then `print(a.drop(2)[2], a.take(10).len)`
Yes, it prints `3 5`. Slices keep their keys, and `take` is clamped.
(R07.1.4, R07.4.11)

Q36. `let s = "abcdef"` then `print(s[2..5][3])`
Yes, it prints `d`. `s[2..5][3]` is `s[3]`. (R07.1.4)

Q37. `print(yes(3 in [1, 2, 3]))`
No: E0606 missing_impl. `in` needs `Contains`, which is only for sub-linear
membership. (R12.4.7)

## Traits

Q38. `fn f(x: Int) -> Int = x` and `fn f(x: Str) -> Int = 0`
No: E0702 overload_by_type. (R06.1.6, R08.5.2)

## Contexts, checks and I/O

Q39. `print(1)`, then `with io(sep: ",")`, then `print(1, 2)`
No: E0802 with_not_first. A bare `with` line must come first in its block.
(R09.1.1)

Q40. The program is `print(true)`.
No: E0606 missing_impl. `Bool` is not `Print`, and the error names
`print(yes(b))`. (R10.3.3)

Other questions I answered while writing, not in the forty:

- `let x: ?Int = 3` then `print(x)`. No: E0606. (R10.5)
- `print(1)`, `print(2)`, `return`, `print(3)`. Yes, it prints `1` and `2`
  on two lines. (R10.1.2, R13.1.3)
- `with io(base: 1)`, `let n = 4`, `domain E = [n]`,
  `let e = "1".parse<E>()!`, `print(Int(e), e, n)`. Yes, it prints `0 1 4`.
  (R10.2.2, R10.3.2)
