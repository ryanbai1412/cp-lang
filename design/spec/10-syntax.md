# 10. Syntax

**Layer:** core C1. Status: [proposed].

## 10.1 Lexical structure

**Source.** UTF-8. Non-ASCII bytes are legal only inside string literals and comments. `//` starts a comment to
end of line; there are no block comments.

**Layout.**
- A logical line ends at a newline outside brackets. Inside `( )` and `[ ]` newlines and indentation are
  ignored, except after a block lambda's `=>` (below).
- Indentation is spaces only. A tab anywhere in leading whitespace is a lex error ("tabs aren't allowed in
  indentation") [2impl Q6].
- A line indented deeper than the previous logical line emits INDENT; a shallower line emits one DEDENT per
  closed level and must land exactly on an enclosing level ("dedent doesn't match any outer block"). The
  indent width is free but consistent within a block.
- **Blocks** open only after a line ending in `:`. A one-line body after `:` or `=` closes that block, so a
  deeper line after it is a lex error ("unexpected indent") [2impl Q4].
- **Continuation:** a line indented deeper than the previous logical line, where the previous line doesn't end in
  `:`, continues it when its first token is `.`, `??`, `and`, `or`, or a binary operator. Anything else there is
  "unexpected indent". This is how method chains and `?? fallback` lines wrap outside brackets [2impl Q11].
- `;` separates simple statements on one logical line (`if c: a.push(x); continue`). It never follows a block
  header or ends a line. The formatter splits `;` lines that exceed the line width [2impl Q5].
- **Block lambdas:** when `=>` ends a line inside brackets, layout resumes for the lambda body: the next line
  opens an INDENT, and the body ends at the matching DEDENT. Bracket mode then resumes.
- Trailing commas are allowed in every comma-separated list [2impl Q11].

**Identifiers.** `[A-Za-z_][A-Za-z0-9_]*`. Types, domains, traits and type parameters start uppercase; every value
binding, function, field and enum variant starts lowercase. A binding with an uppercase name is an error
("values are lowercase; `N` would read as a type") [2impl Q1]. In an expression, `Ident<` starts generic
arguments only when `Ident` is capitalized, so `a < b` and `SegTree<V, M>(…)` need no turbofish.

**Keywords:** `and as break continue domain elif else enum false fn for if impl in let match none not or repeat
return self Self struct trait true type var where while with`, plus area-dependent ones marked in 10.6.

**Literals.**
- Integers: decimal with `_` separators, `0x…`, `0b…`. An `e` literal follows 11.3.
- Floats: `1.5`, `.5` is an error (write `0.5`).
- Bytes: `'c'` is exactly one byte or one escape (`\n \t \r \0 \\ \' \xHH`); `'ab'` is a lex error ("a byte literal
  is one byte; use \"ab\" for text"). There is no `Char` type [2impl Q8].
- Strings: `"…"` with the same escapes plus `\"`, `{expr}` interpolation, `{{` and `}}` for braces.
- `true`, `false`, `none`; list literals `[a, b]`.

## 10.2 Ranges

`a .. b` is half-open and `a ..= b` inclusive (Rust spelling). One-sided forms: `a ..`, `.. b`, `..= b`.
A range is a value (`Range<T>`, library) and the type of a range-typed integer (`lo..=hi`, core).

## 10.3 Precedence and evaluation order

- **Precedence**, tightest first: postfix (`.f`, `[i]`, `!`, `?`, calls); `**` (right-associative; its right operand may start with unary `-`, so `2 ** -1` parses and then traps, 11.4) [2impl Q10]; unary `-` `~`; `* / %`; `+ -`;
  `<< >>`; `&`; `^`; `|`; ranges `.. ..=`; comparisons (non-associative); `not`; `and`; `or`; `??` (right-associative); assignments. So `-x ** 2` is
  `-(x ** 2)` and `not a == b` is `not (a == b)` [r13 P3, P4].
  So `s & (s << 1) == 0` means `(s & (s << 1)) == 0`.

- **Evaluation order** [p1 S4]: operands and arguments evaluate left to right; array `map` and `for` run in
  index order, so a lambda may mutate captures and the effect order is defined.

## 10.4 Patterns

`_` is legal in `let` patterns; tuple patterns may appear in `let`, `for`, lambda and typed `fn`
parameters; destructuring assignment `(a, b) = (x, y)` works on `var`s.

## 10.5 Sugar table

| written | means | chapter |
|---|---|---|
| `domain V = e` | `let (domain V, _) = Domain.of(e)` | 12 |
| `labels(a, b)` | payload-free enum deriving `Index` | 11 |
| `"? {v}"` | `Print` calls into the output buffer | 25 |
| `x op= y` | `x = x op y` (evaluating `x`'s place once) | 16 |
| `for x in s where c:` | `for x in s: if not c: continue` | 15 |
| `f(x?)`, `a[x?]`, `x?.f` | `if let v = x: f(v) else: none` | 23 |

## 10.6 Grammar

EBNF over the token stream. `NL` is the end of a logical line; `INDENT`/`DEDENT` as above. `block := ':' (simple_stmts NL | NL INDENT stmt+ DEDENT)`.
Productions tagged `[area N]` depend on a pending decision in CORE-ALIGN.md; their current form is shown.

```ebnf
file        := item*
item        := decorated | fn_decl | struct_decl | enum_decl | trait_decl | impl_decl | type_decl
             | with_line [area 3] | stmt                                   // a wrap is a stmt
decorated   := ('@' path call_args? NL?)+ (fn_decl | struct_decl | enum_decl | trait_decl | impl_decl)   [area 1]

fn_decl     := 'fn' NAME generics? '(' params? ')' ('->' type)? where? (block | '=' expr NL)
params      := param (',' param)* ','?
param       := ['var' [area 7]] pattern (':' type)? ('=' expr)?          // self / var self in methods
generics    := '<' gparam (',' gparam)* ','? '>'
gparam      := TYPE_NAME ('...')? (':' bound)?                           // '...' = variadic [area 9]
bound       := type ('+' type)*
where       := 'where' TYPE_NAME ':' bound (',' TYPE_NAME ':' bound)*

struct_decl := 'struct' TYPE_NAME generics? ('(' fields? ')')? NL      // no parens: opaque, fields private to its module
fields      := NAME ':' type ('=' expr)? (',' NAME ':' type ('=' expr)?)* ','?
enum_decl   := 'enum' TYPE_NAME generics? ':' (variants NL | NL INDENT (variant NL)+ DEDENT)
variants    := variant (',' variant)* ','?
variant     := NAME ('(' (type | fields) (',' type)* ')')?
trait_decl  := 'trait' TYPE_NAME generics? ('(' bound ')')? ':' NL INDENT trait_item+ DEDENT
trait_item  := 'type' TYPE_NAME (':' bound)? NL | NAME ':' type NL | fn_sig NL | fn_decl
impl_decl   := 'impl' generics? (type 'for')? type where? ':' NL INDENT (fn_decl | 'type' TYPE_NAME '=' type NL | NAME '=' expr NL)+ DEDENT
type_decl   := 'type' TYPE_NAME generics? '=' (type | expr) NL               // expr: fresh type, e.g. Mint(mod: p)

stmt        := simple_stmts NL | compound | wrap
wrap        := '---' 'with' (pattern '<-')? postfix_call '---' NL stmt*        // rest of the block; 16.3 [area 2]
simple_stmts:= simple (';' simple)*
simple      := let | assign | expr | 'return' expr? | 'break' LABEL? | 'continue' LABEL?
let         := ('let' | 'var') pattern (':' type)? '=' expr
             | 'let' pattern '=' expr 'else' block                          [area 6]
             | 'domain' TYPE_NAME '=' expr
             | 'let' '(' 'domain' TYPE_NAME (',' pattern)* ')' '=' expr
assign      := place ('=' | '+=' | '-=' | '*=' | '/=' | '%=' | '&=' | '|=' | '^=' | '<<=' | '>>=' | 'max=' | 'min=') expr
compound    := if_stmt | for_stmt | while_stmt | repeat_stmt | match_stmt | fn_decl | decorated | with_block [area 3]
if_stmt     := 'if' cond block ('elif' cond block)* ('else' block)?
cond        := expr | 'let' pattern '=' expr | place ('max=' | 'min=') expr   // last form [area 7]
for_stmt    := (LABEL ':')? 'for' pattern 'in' expr ('where' expr)? block
while_stmt  := (LABEL ':')? 'while' cond block
repeat_stmt := 'repeat' expr block
match_stmt  := 'match' expr ':' NL INDENT (arm NL)+ DEDENT
arm         := pattern ('if' expr)? '=>' (expr | NL INDENT stmt+ DEDENT)

pattern     := '_' | NAME | literal | '(' pattern (',' pattern)* ','? ')'
             | '.'? NAME '(' pattern (',' pattern)* ')'                      // enum variant
             | literal ('..' | '..=') literal
type        := '?' type | '[' type ']' type | '(' type (',' type)* ','? ')' | path generic_args?
             | expr_range                                                    // lo..hi, lo..=hi: range type
             | ['pure'] '(' (type (',' type)*)? ')' '->' type                // function type [area 7]

expr        := lambda | if_expr | binary
lambda      := lparams '=>' (expr | NL INDENT stmt* expr NL DEDENT)
lparams     := NAME | '_' | '(' (pattern (',' pattern)*)? ')'
if_expr     := 'if' cond ':' expr ('elif' cond ':' expr)* 'else' ':' expr    // else required [2impl Q12]
binary      := operators of 10.3 over unary
unary       := ('-' | '~' | 'not') unary | power
power       := postfix ('**' unary)?
postfix     := primary ( '.' NAME | '.' INT | call_args | '[' expr ']' | '!' | '?' [area 6] )*
call_args   := '(' (arg (',' arg)*)? ','? ')'
arg         := expr | NAME ':' expr                                          // positional before named
primary     := literal | NAME | path generic_args? | '(' expr ')' | tuple | list | string
             | '.' NAME                                                      // variant, type from context [area 10]
```

**Call arguments** [2impl Q64]: positional arguments come first; named ones follow in any order; each parameter is
given at most once; omitted parameters need defaults.
