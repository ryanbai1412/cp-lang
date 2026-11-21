# 15. Control flow

**Layer:** core C9.

Status: [proposed].

**Why.** Python-shaped control flow with one guard keyword (`where`), exhaustive `match`, and `Never`-typed jumps so `?? return x` composes. Everything here is core.

## 15.1 Control flow

- `if c: … elif d: … else: …` [you, p4]; one conditional expression `if c: x elif d: y else: z`. `elif`
  keeps chains at one indent level; `else: if` is legal but the formatter rewrites it to `elif`.
- `for x in seq where cond:`; `while cond:`; `while let x = opt where cond:`; `loop:` with `break` /
  `continue`. `where` is the one guard keyword. Loops may be labeled (`outer: for …`) and `break outer` /
  `continue outer` exit them [r11 N18].
- `if x min= y:` / `while x max= y:`: a `min=`/`max=` that is the **whole** condition yields whether `x`
  changed (a `none` target always does; a lifted `none` right side and an equal value don't). Nowhere else is
  an assignment an expression [r16 R13].
- `if let a = x, b = y:` binds left to right and stops at the first `none` [r11 N11].
- `return`, `break`, `continue` and `fail` are expressions of type `Never`, which converts to any type, so
  `?? return List.filled(n, 0)` works [r11 N5].
- `repeat n:` runs n times; a negative `n` traps [r10 38].
- `match` is exhaustive. It works on enums, literals, tuples and strings; strings need a catch-all arm
  (`_` or a name, as in `other =>`) [r11 N8].
- `return` from nested functions returns from that function only.

- **Grammar** [p2 S3, S12]: a `match` on a range-typed scrutinee is exhaustive when literal arms cover the range,
  and a byte-labelled domain matches on its literals (`'I' =>`). `_` is legal in `let` patterns. `return` at the
  top level ends the program, and below a wrap line returns from its closure, i.e. ends the test (16.3); a lint fires when a `read` is
  reachable after it, naming "read the rest first" [r14 T9]. `assert(cond)` checks an invariant;
  `with checks(asserts: off)` removes it [p2 F9]. Destructuring assignment `(a, b) = (x, y)` works on `var`s.
- **Ordering** [p2 S2]: tuples and structs whose fields are all `Ord` order lexicographically, so
  `best max= (value, who)` is the argmax. A struct widens field-wise when each field widens [p2 S13].
- A dropped non-`Never` result lints unless the callee is `@droppable` (chapter 14) [r13 P6, p4].
