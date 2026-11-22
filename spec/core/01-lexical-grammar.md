# 1. Lexical structure and grammar

Status: draft for 0.3.0. Sources: decisions 10 (archived chapter
`design/spec/10-syntax.md`), 20, 22, 70, 93, 98, 111, 124-137, 140.
Where those disagree, the later decision wins and this chapter states the
result.

## 1.1 Source text

A source file is UTF-8. Bytes outside ASCII may appear only inside string
literals and comments; anywhere else they are a lex error. `Char` and
`Str` values are ASCII (decision 1.6), so a non-ASCII byte inside a
`Char` or `Str` literal is an error too; only comments may carry other
text. (R01.1.1)

- `//` starts a comment that runs to the end of the line. There are no
  block comments. `#` is not a comment; the lexer says so and offers the
  fix. (R01.1.2)
- `///` starts a doc comment attached to the next item. Doc comments are
  Markdown; the first sentence is the summary shown by hover and
  `cpl doc`. Labelled lines after the prose (`cost:`, `traps:`,
  `example:`, `empty:`, `ties:`, `order:`) state the facts every public
  library function documents (`spec/std/README.md`). (R01.1.3)

## 1.2 Layout

Blocks are indentation-based and opened by a line ending in `:`.

```
fn dist(a: P, b: P) -> Int:
    abs(a.x - b.x) + abs(a.y - b.y)
```

- Indentation is spaces only; a tab in leading whitespace is an error.
  The width is free but must be consistent inside one block, and a
  dedent must land exactly on an enclosing level. (R01.2.1)
- A logical line ends at a newline outside brackets. Inside `( )`,
  `[ ]` and `{ }` newlines and indentation are ignored, with one
  exception: a line that ends in `=>` (a block lambda) or in `:` (a
  bracketed `match`, `if` or `else` block) resumes layout until the
  matching dedent (decision 98). (R01.2.2)
- A one-line body follows `:` or `=` on the same line. A deeper line
  after a one-line body is "unexpected indent", unless it is a
  continuation line (below), which extends that body. (R01.2.3)
- **Continuation.** A line indented deeper than the previous logical
  line, where that line does not end in `:`, continues it when its first
  token is `.`, `??`, `and`, `or` or a binary operator. This is how
  method chains and fallbacks wrap without brackets: (R01.2.4)

  ```
  let best = xs.filter(p)
      .map(f)
      .max() ?? 0
  ```

- `;` separates simple statements on one line
  (`if c: a.push(x); continue`). It never ends a line or follows a block
  header. `{ ... }` is an inline expression block whose statements are
  separated by `;` and whose value is its last expression. (R01.2.5)
- Trailing commas are allowed in every comma-separated list. (R01.2.6)

## 1.3 Names

Identifiers are `[A-Za-z_][A-Za-z0-9_]*`. Case is part of the grammar:

- Types, domains, traits, type parameters and enum variants are
  `PascalCase`: `Some(x)`, `None`, `Ev.Add(v)`, `Dir.L` (decision 157).
  `true` and `false` are lowercase literals. (R01.3.1)
- Values, functions, fields and modules are `snake_case` or lower camel;
  a value binding with an uppercase first letter is an error ("values are
  lowercase; `N` would read as a type"). (R01.3.2)
- So a pattern needs no lookup to read: a capitalized name is a variant
  or constant, a lowercase name binds. `match h: Big => ...` tests the
  variant; `match h: big => ...` binds `big`. (R01.3.3)
- In an expression, `Name<` opens type arguments when `Name` is
  capitalized and what follows parses as type arguments closed by `>`,
  and `name<` (lowercase) when the closing `>` is immediately followed by
  `(`, as in `s.parse<Mod<P>>()`.
  So `a < b` is a comparison and `SegTree<0..=4e9>(+, a)` needs no
  turbofish. A lowercase generic function takes its type arguments from
  the expected type, or explicitly as `read<[n]Int>()`. (R01.3.4)
- `_` is the discard name in patterns and parameters. (R01.3.5)

### Reserved words

The 33 keywords are:

<!-- spec-only: keyword list, not a program -->
```
let var const fn struct enum domain type use pub if elif else for while
in match return break continue and or not assert mut self unchecked
true false trait impl lends yield
```

`todo` is a primary expression and `context`, `marker` and `uses` are
contextual words (R11.1.2, R09.1.3): each is a keyword only where the
grammar expects it and an ordinary name elsewhere.

`Some` and `None` are not reserved: they are the variants of `?T`, in
the prelude like any other name, so a user enum may also have a `None`
variant (`Side.None`). (R01.3.6)

Contextual words are keywords only in the position shown and ordinary
names elsewhere:

| Word | Position |
|---|---|
| `is` | `x is pat` in value position (decision 111) |
| `with` | `with ctx(...):` blocks and `@with` (chapter 9); record update `x with f: v` |
| `yields` | `-> yields T` on a generator fn (chapter 6) |
| `law` | inside a `trait` or `impl` body (chapter 8) |
| `where` | trait bounds after a signature (chapter 8) |
| `without` | row types `..R without (u, v)` (chapter 8) |
| `mod`, `extern` | inline modules and judge-provided fns (chapter 11) |
| `by` | range steps `0..n by 2` |
| `of` | constrained strings `Str of "01?"` (chapter 2) |
| `Self` | inside `trait` and `impl` bodies |

`unchecked:` opens a block whose arithmetic and indexing checks are off
in every build (chapter 9); it is a statement, never an expression. (R01.3.7)

## 1.4 Literals

- **Integers**: decimal with `_` separators, `0x...`, `0b...`.
  Scientific literals that denote an exact integer are `Int`:
  `2e5`, `1e9 + 7`, `4e18`. A literal's type is the singleton range
  `v..=v`, which widens as arithmetic proceeds (chapter 3). (R01.4.1)
- **Floats** need a fraction or a non-integer value: `0.5`, `1.5e-9`.
  `.5` and `5.` are errors. (R01.4.2)
- **Chars**: `'c'`, exactly one ASCII character or one escape
  (`\n \t \r \0 \\ \' \xHH`, `HH <= 7F`). `'ab'` is an error that
  suggests `"ab"`. (R01.4.3)
- **Strings**: `"..."` with the same escapes plus `\"`. Every string
  literal interpolates: `"Case #{t}: {ans}"`; `{{` and `}}` are literal
  braces; `r"..."` is raw (no escapes, no interpolation). Format specs
  follow a colon inside the braces: `"{x:.9}"`, `"{v: }"` (chapter 10). (R01.4.4)
- `true`, `false`, `None`.
- **Lists** `[a, b, c]`, **maps** `[k: v, k2: v2]` (`[:]` is empty),
  **tuples** `(a, b)`, **records** `(lo: 1, hi: n)`. (R01.4.5)
- **Literal-union types** use literals as types: `'+' | '-'`,
  `"add" | "del"` (decision 135).

`++` is a lex error ("CP Lang concatenates with `+`"). (R01.4.6)

## 1.5 Operators and precedence

Tightest first:

1. postfix: `.f`, `.0`, calls `f(...)`, indexing `a[i]`, `!` (unwrap or
   trap), `?` (propagate `None`), `is pattern` (so `a + b is P` is
   `a + (b is P)`; parenthesize the left side)
2. `**` (right-associative; the right operand may start with unary `-`)
3. unary `-`, `+` (numeric only), `~`
4. `*`, `/`, `%`
5. `+`, `-`
6. `<<`, `>>`
7. `&`
8. `^` (xor only; power is `**`)
9. `|`
10. ranges `..`, `..=` (and the one-sided forms)
11. comparisons `< <= > >= == != in`, chainable in one direction only
12. `not`
13. `and`
14. `or`
15. `??` (right-associative, lazy right side)
16. lambdas `=>`, inline `if ... else ...`

So `-x ** 2` is `-(x ** 2)`, `not a == b` is `not (a == b)`, and
`s & (s << 1) == 0` is `(s & (s << 1)) == 0`. (R01.5.1)

- `/` is floor division and `%` is never negative (decision 24). (R01.5.2)
- Chained comparisons keep Python's meaning, each operand evaluated
  once, but only when every link points the same way: `0 <= i < j <= n`
  is legal; `a < b > c` and any chain containing `==` or `!=` is an error
  suggesting `and` (decision 130). (R01.5.3)
- Operands and arguments evaluate left to right. Assignment evaluates
  the right side first, then the place (chapter 5). (R01.5.4)
- `and`, `or`, `+`, `*`, `min`, `max`, `gcd`, `^`, `&`, `|` are also
  operation values (`xs.fold(or)`, `SegTree(max, a)`; `lib/ops-ranges.md`).
  `add` is the prelude name of `+` as a value: the same value, so `act: +`
  and `act: add` are one argument. (R01.5.5)
- `??` written next to `+ - * / %` without parentheses is an error:
  `a ?? 0 + 1` must be written `(a ?? 0) + 1` or `a ?? (0 + 1)`, since
  either reading is a common slip. (R01.5.6)

## 1.6 Statements

- `let p = e` binds immutably; `var p = e` mutably; `const N = e` is a
  compile-time constant. `let p: T = e` annotates. Several names may
  share a type: `let n, m: 1..=2e5 = read()`; `var a, b: Int = 0` fills
  each with the value. (R01.6.1)
- `let p = e else: block` binds or runs the block, which must leave
  (`return`, `break`, `continue` or a trap). (R01.6.2)
- `var` inside a pattern makes that one binding mutable:
  `for (var left, v) in items:`. (R01.6.3)
- Assignment `place = e`, destructuring `(a, b) = (b, a)`, compound
  assignment `x op= e` for every binary operator and for `min=` /
  `max=`. Compound assignment is a statement, except that `min=` and
  `max=` used as a condition are `true` when the target strictly
  improved, including from `None` (decision 129):
  `if dist[v] min= dist[u] + w: parent[v] = u`. (R01.6.4)
- `return e` exists for early exits; a block's value is its last
  expression. At top level `return` ends the program (or the test case
  under `@!multitest`). (R01.6.5)
- `assert cond` and `assert cond, "message {x}"`: checked builds only,
  unless opted in for release (chapter 9). (R01.6.6)
- `todo` is an expression of any type that traps when run; library
  stubs use it as a body. (R01.6.7)

## 1.7 Control flow

<!-- spec-only: syntax tour with `...` placeholder bodies -->
```
if c: a elif d: b else: e           // inline form is an expression
if let v = m.get(k), v > 0: ...     // header binders, comma conditions
while let x = q.pop(): ...
for v in V: ...
for i in V, j in V.after(i) if ok(i, j): ...   // clauses nest; `if` filters
'outer: for r in R:
    for c in C:
        if g[r, c] == '#': break 'outer
else:
    print("none")                   // loop else: runs when no break
match op:
    Add(l, r) => st.apply(l..r, add(1))
    Ask(i) =>
        print(st[i])
match d: L => -1, R => 1            // one-line match
```

- `if let` / `while let` bind in headers; `x is pat` (with `and cond`) is
  the value-position form. `if x is p:` in a header gets a lint with a
  fix to the `let` form (decision 111). (R01.7.1)
- `for` and `while` take an `else:` that runs when the loop ended
  without `break`; a labelled break from an inner loop skips the inner
  `else` (decision 125). Labels are `'name:` before the loop. (R01.7.2)
- `match` is exhaustive over enums, literal unions, `Bool` and `?T`.
  Arms may have guards `pat if cond =>`. There are no custom patterns:
  a view is a method returning an enum (decision 132). (R01.7.3)
- Loops over library generators support `break`, `continue`, `return`,
  labels and `else` exactly like built-in loops (decision 126). (R01.7.4)
- `@!multitest` on its own line is the multitest divider: code above it
  runs once, code below runs per test case (decision 20; chapter 10). (R01.7.5)

## 1.8 Expressions

- **Lambdas** `x => e`, `(a, b) => e`, `(d: Int, l: Low) => e`, `_ => e`,
  `(mut d, e) => d.unite(e.0, e.1)` (a `mut` parameter, as in `fn`),
  and block lambdas: a line ending in `=>` opens an indented body whose
  value is its last expression. A lambda body may be an assignment,
  and then returns nothing. A `let`-bound lambda may call itself
  (decision 127). (R01.8.1)
- **Comprehensions** `[f(x) for x in xs if p(x)]` build a `List`, or an
  array when the expected type is `[D]T`. **Tables** `[D] i => e` build
  `[D]T` eagerly; `[(A, B)] (a, b) => e` and `[A, B]` are the same 2-D
  table. (R01.8.2)
- **Record update** `e with f: v, g: w` and `(..e, f: v)` copy `e` with
  some fields replaced (decision 134, row types in chapter 8). (R01.8.3)
- **Ranges** `a..b`, `a..=b`, `a..`, `..b`, `..=b`, `..` (the full range
  wherever a range is taken), steps `a..b by k`. (R01.8.4)
- **Indexing** `a[i]`, `g[r, c]` (never `g[r][c]`); `a[i]` on `[I, J]T`
  is the row place. `a[l..r]` is a slice view keeping the source keys. (R01.8.5)
- **Optional chaining**: `x?.f`, `f(x?)` propagate `None`; `x ?? y`
  falls back lazily; `x!` unwraps or traps with the source location. (R01.8.6)
- Members without parentheses are O(1) (`xs.len`, `t.parent[v]`);
  anything that does work has parentheses (`g.transpose()`), decision 140. (R01.8.7)

## 1.9 Items

<!-- spec-only: syntax tour over undeclared names and `todo` bodies -->
```
use std.segtree.{SegTree}                 // imports; prelude is implicit
pub fn f<T: Ord>(xs: [D]T, by: T -> K = id) -> ?D = todo
struct Edge: u: V, v: V, w: Int           // one-line struct
struct Tally:
    var top = 0                           // var fields may change
    total: Int = 0
    fn add(mut self, x: Int):
        self.top max= x
        self.total += x
enum Dir = L | R | U | D
enum Ev:
    Add(v: V)
    Ask(l: V, r: V)
trait Monoid<T>(Op<T>):
    @get fn id(self) -> T
    law left_id(x: T) = self.op(self.id, x) == x
impl Monoid<Mat> for MatMul: ...
type Grid = [R, C]Char
domain V = 1..=n                          // a fresh index type
@memo(array) fn ways(i: V.cuts) -> Mod<P>: ...
mod geometry: ...                         // inline module
extern fn ask(i: Int) -> Int              // provided by the judge
```

Decorators are `@name` or `@name(args)` on the next item; the set is
closed and listed in §6.4, plus `@!multitest` as the one file directive. Types, domains and structs may be declared inside a
function body and use its runtime domains (decision 136). (R01.9.1)

## 1.10 Grammar

EBNF over the token stream. `NL` ends a logical line; `INDENT` and
`DEDENT` come from layout. `block := ':' (simple_stmts NL | NL INDENT
stmt+ DEDENT)`. On one line, `if c: s1 else: s2` (and `elif`) with simple
statements is an `if_stmt`: the `NL` before `else` is optional when the
whole statement fits on one line. In a type position, `[D]` is followed by
a type, so `[V]-1e9..=1e9` is a range type whose ends may be signed.

<!-- spec-only: grammar, not CP code -->
```ebnf
file        := (directive | item)*
directive   := '@!' NAME call_args? NL
item        := decorator* (fn_decl | struct_decl | enum_decl | trait_decl
             | impl_decl | type_decl | domain_decl | mod_decl | extern_decl
             | use_decl) | stmt
decorator   := '@' path call_args? NL?
use_decl    := 'pub'? 'use' path ('.' '{' NAME (',' NAME)* ','? '}')? NL

fn_decl     := 'pub'? 'fn' fn_name generics? '(' params? ')' ret? where?
               (block | '=' expr NL)
fn_name     := NAME | '(' param ')' BINOP '(' param ')'     // operator fns
ret         := '->' ('yields' | 'lends' lend_src?)? type
lend_src    := '(' NAME (',' NAME)* ')'
params      := param (',' param)* (';' param (',' param)*)? ','?
param       := 'mut'? (pattern | 'self') (':' type)? ('=' expr)?
             | '..' NAME ':' TYPE_NAME                      // variadic tuple
generics    := '<' gparam (',' gparam)* ','? '>'
gparam      := 'domain'? TYPE_NAME (':' bound)? ('=' type)?
             | 'const' NAME ':' type
bound       := type ('+' type)*
where       := 'where' TYPE_NAME ':' bound (',' TYPE_NAME ':' bound)*

struct_decl := 'pub'? 'struct' TYPE_NAME generics? (':' fields NL
             | ':' NL INDENT (field NL | fn_decl)+ DEDENT | NL)  // bodyless: opaque
fields      := field (',' field)*
field       := 'var'? NAME (':' type)? ('=' expr)?
enum_decl   := 'pub'? 'enum' TYPE_NAME generics? ('=' NAME ('|' NAME)* NL
             | ':' NL INDENT (variant NL | fn_decl)+ DEDENT)
variant     := decorator* NAME ('(' fields ')')?
trait_decl  := 'pub'? 'trait' TYPE_NAME generics? ('(' bound ')')? ':'
               NL INDENT trait_item+ DEDENT
trait_item  := 'type' TYPE_NAME (':' bound)? NL | field NL | fn_sig NL
             | fn_decl | law
law         := 'law' NAME '(' params? ')' '=' expr NL
impl_decl   := 'impl' generics? bound ('for' type)? where? (':' NL INDENT
               (fn_decl | 'type' TYPE_NAME '=' type NL | law)+ DEDENT | NL)
type_decl   := 'pub'? 'type' TYPE_NAME generics? '=' type NL
domain_decl := 'domain' TYPE_NAME '=' expr NL
             | 'let' '(' 'domain' TYPE_NAME (',' pattern)* ')' '=' expr NL
mod_decl    := 'pub'? 'mod' NAME ':' NL INDENT item+ DEDENT
extern_decl := 'extern' 'fn' NAME '(' params? ')' ret? NL

stmt        := simple_stmts NL | compound
simple_stmts:= simple (';' simple)*
simple      := let | assign | expr | 'return' expr? | 'break' LABEL?
             | 'continue' LABEL? | 'assert' expr (',' expr)?
let         := ('let' | 'var' | 'const') pattern (',' NAME)* (':' type)?
               '=' expr ('else' block)?
assign      := place ('=' | ASSIGN_OP | NAME '=') expr     // `x min= y`, `x f= y`
compound    := if_stmt | for_stmt | while_stmt | match_stmt | with_stmt
             | unchecked | decorator* fn_decl
if_stmt     := 'if' cond block ('elif' cond block)* ('else' block)?
cond        := 'let' pattern '=' expr (',' cond)* | expr (',' cond)*
             | place ('min=' | 'max=') expr      // true when improved (R01.6.4)
for_stmt    := (LABEL ':')? 'for' clause (',' clause)* block ('else' block)?
clause      := pattern 'in' expr ('if' expr)?
while_stmt  := (LABEL ':')? 'while' cond block ('else' block)?
match_stmt  := 'match' expr ':' (arm (',' arm)* NL | NL INDENT (arm NL)+ DEDENT)
arm         := pattern ('if' expr)? '=>' (expr | assign | NL INDENT stmt+ DEDENT)
with_stmt   := 'with' call (',' call)* block
unchecked   := 'unchecked' block

pattern     := '_' | 'var'? NAME | literal | range_pat
             | '(' pattern (',' pattern)* ','? ')'
             | '(' NAME ':' pattern (',' NAME ':' pattern)* (',' '..')? ')'
             | path ('(' pattern (',' pattern)* ')')?       // variants, V(j)
             | pattern '|' pattern
range_pat   := literal ('..' | '..=') literal
type        := '?' type | '[' dims ']' type | '(' type (',' type)* ','? ')'
             | '(' NAME ':' type (',' NAME ':' type)* (',' '..' TYPE_NAME
               ('without' '(' NAME (',' NAME)* ')')?)? ')'     // records, rows
             | path generic_args? | range_type | literal ('|' literal)*
             | '(' (type (',' type)*)? ')' '->' type          // fn types
             | type 'of' STRING
dims        := type (',' type)* | '(' type (',' type)* ')' | expr
range_type  := expr? ('..' | '..=') expr?

expr        := lambda | if_expr | binary
lambda      := lparams '=>' (expr | assign | NL INDENT stmt* expr? DEDENT)
lparams     := NAME | '_' | '(' (param (',' param)*)? ')' ('->' type)?
if_expr     := 'if' cond ':' expr ('elif' cond ':' expr)* 'else' ':' expr
binary      := the operators of 1.5 over unary; `not` is the prefix
               operator at level 12, so `not a == b` is `not (a == b)`
unary       := ('-' | '+' | '~') unary | power
power       := postfix ('**' unary)?
postfix     := primary ('.' NAME | '.' INT | call_args | '[' args ']'
             | '!' | '?' | 'is' pattern | 'with' NAME ':' expr (',' NAME ':' expr)*)*
call_args   := '(' (arg (',' arg)*)? ','? ')'
arg         := expr | NAME ':' expr | 'mut' place
primary     := literal | NAME | path generic_args? | '(' expr ')' | tuple
             | record | list | map | comprehension | table | '{' block_expr '}'
             | 'todo' | BINOP                                 // `+` as an op value
```

Call arguments: positional first, named after in any order, each
parameter at most once; parameters after `;` are named-only. A `mut`
parameter is marked `mut` at the call too (`link(mut d, a, b)`), except
for the receiver of a method call. (R01.10.1)

## 1.11 Rejected spellings

Each of these gets an error with a machine-applicable fix: (R01.11.1)

| Written | Write instead | Decision |
|---|---|---|
| `#` comment | `//` | 10.1 |
| `xs ++ ys` | `xs + ys` | 98 |
| `for x in xs where p` | `for x in xs if p` | 70, 93 |
| `x.pow(k)`, `x ^ k` meaning power | `x ** k` | 70 |
| `xs.len()` on a getter | `xs.len` | 93 |
| `if x is p:` header | `if let p = x:` | 111 |
| `{k: v}` map literal | `[k: v]` | 135 |
| `[x; n]` | `[x] * n` or a typed `[n]T` | 135 |
| `(let a = ...; e)` | a block lambda or `{ let a = ...; e }` | 135 |
| `fn f() -> T =` then an indented block | `fn f() -> T:` | 136 |
| `--- multitest ---` | `@!multitest` | 93 |
| `defer` and other v2.1 words | as the error says | 19 |
