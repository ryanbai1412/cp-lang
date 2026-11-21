"""Error and lint index catalog and HTML renderer for CP Lang.

Every code in `crates/cpl-diag/src/lib.rs` and `crates/cpl-lint/src/lib.rs`
must appear in `ERROR_CATALOG`. `site/check_docs.py` enforces 100% coverage
and verifies every trigger/fix pair against the real `cpl` binary.
"""

from __future__ import annotations

import html
from pathlib import Path
import re

CATEGORY_ORDER = [
    ("Lexical errors (E00xx)", "E00"),
    ("Syntax errors (E01xx)", "E01"),
    ("Names and modules (E02xx)", "E02"),
    ("Types and domains (E03xx)", "E03"),
    ("Ownership and mutability (E04xx)", "E04"),
    ("Backends and compatibility (E05xx)", "E05"),
    ("Lends, projections and traits (E06xx)", "E06"),
    ("Warnings (W01xx)", "W01"),
    ("Runtime traps (R00xx)", "R00"),
    ("Lints (L0xxx)", "L0"),
    ("Internal compiler error (E9999)", "E99"),
]


def _entry(
    code: str,
    title: str,
    summary: str,
    explanation: str,
    trigger: str,
    fix: str,
    mode: str = "check",
    lint_name: str | None = None,
) -> dict:
    return {
        "code": code,
        "title": title,
        "summary": summary,
        "explanation": explanation,
        "trigger": trigger.strip("\n") + "\n" if trigger else "",
        "fix": fix.strip("\n") + "\n" if fix else "",
        "mode": mode,
        "lint_name": lint_name,
    }


ERROR_CATALOG: list[dict] = [
    # --- E00xx: Lexical ---
    _entry(
        "E0001",
        "Tab character in indentation",
        "Blocks must be indented with spaces, not tabs.",
        "CP Lang uses indentation to delimit blocks, and mixing tabs with spaces makes nesting ambiguous across editors. Replace leading tabs with 4 spaces (or run `cpl fmt`).",
        "if true:\n\tprint(1)",
        "if true:\n    print(1)",
    ),
    _entry(
        "E0002",
        "Inconsistent dedent",
        "A line dedented to a column that does not match any enclosing block.",
        "When closing an indented block, the next line's column must match an outer indentation level exactly. Align the line with its enclosing block.",
        "if true:\n    let x = 1\n  print(x)",
        "if true:\n    let x = 1\n    print(x)",
    ),
    _entry(
        "E0003",
        "Unterminated string literal",
        "A string literal reached the end of the line or file without a closing quote.",
        "String literals must be closed with `\"` on the same line (and escape quotes inside them as `\\\"`).",
        'let s = "hello',
        'let s = "hello"',
    ),
    _entry(
        "E0004",
        "Unknown character",
        "The lexer encountered a character that is not valid in CP Lang source.",
        "Operators like `&&` or stray punctuation such as `$` or `` ` `` are not valid tokens in CP Lang. Use CP Lang keywords (`and`, `or`, `not`) and ASCII operators.",
        "let x = $1",
        "let x = 1",
    ),
    _entry(
        "E0005",
        "Malformed number literal",
        "A numeric literal has an invalid prefix, digit, or exponent.",
        "Binary literals (`0b...`) only allow `0` and `1`, hex literals (`0x...`) only allow `0-9a-fA-F`, and scientific notation requires at least one exponent digit.",
        "let x = 0b102",
        "let x = 0b101",
    ),
    _entry(
        "E0006",
        "Unexpected indentation",
        "A line was indented where no block was opened.",
        "Only statements ending in `:` (such as `if`, `for`, `while`, `fn`, `struct`, `enum`, `match`) start an indented block. Remove the extra leading spaces.",
        "let x = 1\n    let y = 2",
        "let x = 1\nlet y = 2",
    ),
    _entry(
        "E0007",
        "Malformed character literal",
        "A `Char` literal must hold exactly one ASCII byte or valid escape.",
        "Single-quoted literals represent one `Char` (`'a'`, `'\\n'`). Use double quotes `\"...\"` for multi-character strings.",
        "let c = 'ab'",
        'let s = "ab"',
    ),
    # --- E01xx: Syntax ---
    _entry(
        "E0101",
        "Unclosed delimiter",
        "An opening `(`, `[`, or `{` has no matching closing delimiter.",
        "Every opening parenthesis, bracket, or brace must be closed before the end of the expression or file.",
        "let x = (1 + 2",
        "let x = (1 + 2)",
    ),
    _entry(
        "E0102",
        "Expected expression",
        "The parser expected a value or expression at this position.",
        "An operator, assignment, or call argument is missing its operand expression.",
        "let x = 1 +",
        "let x = 1 + 2",
    ),
    _entry(
        "E0103",
        "Expected token",
        "A required punctuation token or keyword (such as `:` or `=`) is missing.",
        "Check the header syntax of the surrounding construct: `if`, `for`, `while`, `fn`, and `match` require `:` before their body.",
        "if true\n    print(1)",
        "if true:\n    print(1)",
    ),
    _entry(
        "E0104",
        "Expected indented block",
        "A header ended with `:` but no indented body followed.",
        "Provide at least one statement on the same line after `:` or in an indented block on the next line.",
        "if true:\nprint(1)",
        "if true:\n    print(1)",
    ),
    _entry(
        "E0105",
        "Expected type",
        "The parser expected a type annotation after `:` or `->`.",
        "Supply a valid type such as `Int`, `Str`, `[V]Int`, or `?Int`.",
        "let x: = 1",
        "let x: Int = 1",
    ),
    _entry(
        "E0106",
        "Expected pattern",
        "The parser expected a binding or match pattern.",
        "Bindings (`let`/`var`) and `match` arms require a valid pattern such as a name, tuple `(a, b)`, enum variant, literal, or `_`.",
        "match 1:\n    => print(1)",
        "match 1:\n    _ => print(1)",
    ),
    _entry(
        "E0107",
        "Unexpected token",
        "An unexpected token appeared where the parser did not allow it.",
        "Check for stray keywords, missing operators, or extra tokens at the end of a statement.",
        "let x = 1 2",
        "let x = 1 + 2",
    ),
    _entry(
        "E0108",
        "Expected identifier",
        "The parser expected a name for a declaration, parameter, or field.",
        "Declarations (`fn`, `struct`, `enum`, `domain`) and member accesses (`.field`) must be followed by a valid identifier.",
        "fn 123():\n    print(1)",
        "fn f():\n    print(1)",
    ),
    _entry(
        "E0109",
        "Unmatched closing delimiter",
        "A closing `)`, `]`, or `}` appeared with no matching opener.",
        "Remove the extra closing delimiter or add the missing opening delimiter.",
        "let x = 1 + 2)",
        "let x = (1 + 2)",
    ),
    _entry(
        "E0110",
        "Unparenthesized `??` with arithmetic",
        "`??` cannot be mixed directly with `+`, `-`, `*`, `/`, or `%` without parentheses.",
        "Mixing null-coalescing `??` with arithmetic is a common source of precedence bugs (`a ?? 0 + 1` vs `(a ?? 0) + 1`). Parenthesize the intended grouping explicitly.",
        "let a: ?Int = 1\nlet b = a ?? 0 + 1",
        "let a: ?Int = 1\nlet b = (a ?? 0) + 1",
    ),
    # --- E02xx: Names & Modules ---
    _entry(
        "E0201",
        "Unresolved name",
        "An identifier was used that is not declared in scope.",
        "Check for typos, missing `use ...` imports, or variables declared inside an inner block that went out of scope.",
        "print(not_defined)",
        "let defined = 1\nprint(defined)",
    ),
    _entry(
        "E0202",
        "Duplicate definition",
        "Two declarations in the same scope share the same name.",
        "Rename one of the declarations, or update an existing `var` without redeclaring it.",
        "fn f() -> Int: 1\nfn f() -> Int: 2",
        "fn f() -> Int: 1\nfn g() -> Int: 2",
    ),
    _entry(
        "E0203",
        "Unknown field or method",
        "A struct, tuple, or type has no field or method with that name.",
        "Check the field name on the struct definition or the available methods for the receiver's type.",
        "struct Pt(x: Int, y: Int)\nlet p = Pt(1, 2)\nprint(p.z)",
        "struct Pt(x: Int, y: Int)\nlet p = Pt(1, 2)\nprint(p.x)",
    ),
    _entry(
        "E0204",
        "Unknown type",
        "A type annotation names a type or domain that is not in scope.",
        "Check the spelling of the type, declare the `struct`/`enum`/`domain` first, or import it with `use`.",
        "let x: DoesNotExist = 1",
        "let x: Int = 1",
    ),
    _entry(
        "E0205",
        "Used before initialization",
        "A top-level value was referenced before the line that defines it (or inside its own initializer).",
        "Top-level statements execute in order. Move the definition above its first use.",
        "print(x)\nlet x = 1",
        "let x = 1\nprint(x)",
    ),
    _entry(
        "E0206",
        "Unresolved module",
        "A `use` statement names a module that could not be found.",
        "Check the module path (`use std.graph.{...}`, `use std.tree.{...}`, `use std.segtree.{...}`) or your `cp.toml` library configuration.",
        "use std.no_such_module.{foo}",
        "use std.math.{gcd}\nprint(gcd(12, 18))",
    ),
    _entry(
        "E0207",
        "Private item",
        "A `use` statement tried to import an item that is not marked `pub`.",
        "Only items declared `pub` are part of a module's public API. Import the public entry point instead.",
        "use std.strings.{sa_is}",
        'use std.strings.{z_function}\nprint(z_function("ababa"))',
    ),
    # --- E03xx: Types & Domains ---
    _entry(
        "E0301",
        "Wrong index domain",
        "An array or grid was indexed with a position from a different domain.",
        "Nominal domains (`domain Row`, `domain Col`) are distinct types even when they have the same size. Index each axis with its own domain value.",
        "domain Row = 1..=3\ndomain Col = 1..=3\nlet a: [Row]Int = 0\nprint(a[Col.first])",
        "domain Row = 1..=3\ndomain Col = 1..=3\nlet a: [Row]Int = 0\nprint(a[Row.first])",
    ),
    _entry(
        "E0302",
        "Type mismatch",
        "An expression's type does not match the expected type.",
        "Convert explicitly (for example `F64(n)`, `Int(ch)`, or `floor(x)`) or fix the type annotation.",
        'let x: Int = "abc"',
        "let x: Int = 123",
    ),
    _entry(
        "E0303",
        "Wrong number of arguments",
        "A function or constructor was called with too few or too many arguments.",
        "Match the parameter count of the called function or struct constructor.",
        "fn add(a: Int, b: Int) -> Int: a + b\nprint(add(1))",
        "fn add(a: Int, b: Int) -> Int: a + b\nprint(add(1, 2))",
    ),
    _entry(
        "E0304",
        "Value is not callable",
        "Call syntax `x(...)` was used on a value that is not a function or type.",
        "Remove the parentheses if you meant to read a variable, or call a function value.",
        "let x = 5\nprint(x(1))",
        "let x = 5\nprint(x)",
    ),
    _entry(
        "E0305",
        "Value is not indexable",
        "Indexing `x[i]` was used on a type that does not support indexing.",
        "Only arrays (`[D]T`), lists (`List<T>`), strings (`Str`), maps (`Map<K, V>`), and types implementing `Index` can be indexed.",
        "let x = 5\nprint(x[0])",
        "let a = [5]\nprint(a[0])",
    ),
    _entry(
        "E0306",
        "Invalid arithmetic on domain position (reserved)",
        "Reserved diagnostic code for disallowed arithmetic on domain positions (currently reported via `E0309` and lint `L0101`).",
        "Use `v + k` / `v - k` to step by an `Int` offset (returning `?V`), `b - a` for signed distance, or `v.to_idx` for a `0..V.count` integer.",
        "",
        "domain V = 1..=3\nlet x = V.last - V.first\nprint(x)",
        mode="reserved",
    ),
    _entry(
        "E0307",
        "Condition is not `Bool`",
        "An `if`, `elif`, `while`, `assert`, or `where` condition must have type `Bool`.",
        "CP Lang does not coerce integers or optionals to booleans. Write an explicit comparison like `x != 0` or `if let v = opt:`.",
        "let x = 1\nif x:\n    print(1)",
        "let x = 1\nif x != 0:\n    print(1)",
    ),
    _entry(
        "E0308",
        "Ordering comparison on optional",
        "Ordering operators (`<`, `<=`, `>`, `>=`) cannot compare `?T` directly.",
        "Because `none` has no universal ordering (infinity vs negative infinity depends on min vs max), unwrap with `??`, `if let`, or use `min=` / `max=`.",
        "let a: ?Int = 1\nlet b: ?Int = 2\nif a < b:\n    print(1)",
        "let a: ?Int = 1\nlet b: ?Int = 2\nif let x = a, y = b where x < y:\n    print(1)",
    ),
    _entry(
        "E0309",
        "Incompatible operands for operator",
        "An operator was applied to operand types it does not support.",
        "Check the operand types and convert explicitly when mixing `Int`, `F64`, `U64`, `Str`, or `Bool`.",
        'let x = "a" + 1',
        'let x = "a" + "1"\nprint(x)',
    ),
    _entry(
        "E0310",
        "Unknown named argument",
        "A call passed a keyword argument name that the function does not accept.",
        "Check the parameter names in the function signature (for example `sep:`, `end:`, `none:`, `by:`).",
        "fn f(x: Int) -> Int: x\nprint(f(y: 1))",
        "fn f(x: Int) -> Int: x\nprint(f(x: 1))",
    ),
    _entry(
        "E0311",
        "Invalid `@memo` function",
        "A `@memo` function has parameters or mutations that cannot be memoized safely.",
        "`@memo` requires finite-domain or hashable value parameters, a return value, and no `mut` parameters.",
        "@memo\nfn bad(mut x: Int) -> Int:\n    x += 1\n    return x",
        "@memo\nfn fib(n: 0..=20) -> Int:\n    if n <= 1: return n\n    return fib(n - 1) + fib(n - 2)\nprint(fib(10))",
    ),
    _entry(
        "E0312",
        "Mutating expression inside `debug(...)`",
        "`debug(...)` arguments must not mutate program state.",
        "Because `cpl build` strips `debug(...)` from judge C++17, any mutation inside `debug(...)` would cause local runs and judge submissions to diverge. Compute the mutation outside `debug`.",
        "var x = 0\nfn bump() -> Int:\n    x += 1\n    return x\ndebug(bump())",
        "var x = 0\nx += 1\ndebug(x)\nprint(x)",
    ),
    _entry(
        "E0313",
        "Fresh domain escapes its declaring block",
        "A value whose type mentions a local `domain` cannot escape the block or function that declared it.",
        "Declare the domain in an enclosing scope or return it explicitly with an existential return type `-> (domain P, [P]Int)` and unpack it at the call site with `let (domain P, part) = f(..)`.",
        "fn bad(k: Int) -> [0..k]Int:\n    domain Local = 0..k\n    let a: [Local]Int = 0\n    return a",
        "fn good(k: Int) -> (domain P, [P]Int):\n    domain Local = 0..k\n    let a: [Local]Int = 0\n    return (Local, a)\nlet (domain Q, vals) = good(3)\nprint(Q.count, vals[Q(0)])",
    ),
    # --- E04xx: Ownership & Mutability ---
    _entry(
        "E0401",
        "Assignment to immutable `let` binding",
        "A `let` binding cannot be reassigned or mutated.",
        "Declare the binding with `var` if it needs to change.",
        "let x = 1\nx = 2",
        "var x = 1\nx = 2\nprint(x)",
    ),
    _entry(
        "E0402",
        "Missing `mut` at call site",
        "A function parameter declared `mut` must also be marked `mut` at the call site.",
        "Marking `mut` at every call site makes mutations visible when scanning contest code at 3am.",
        "fn inc(mut x: Int):\n    x += 1\nvar a = 0\ninc(a)",
        "fn inc(mut x: Int):\n    x += 1\nvar a = 0\ninc(mut a)\nprint(a)",
    ),
    _entry(
        "E0403",
        "Mutable borrow of immutable value (reserved)",
        "Reserved diagnostic code for passing an immutable binding as `mut` (currently reported via `E0401` / `E0402`).",
        "Change `let` to `var` on the owning binding, or copy it into a local `var` before mutating.",
        "",
        "var a = [1, 2, 3]\na.push(4)\nprint(a)",
        mode="reserved",
    ),
    _entry(
        "E0404",
        "Top-level `var` referenced inside `fn` (reserved)",
        "Reserved diagnostic code for top-level `fn` capturing a mutable top-level `var`.",
        "Pass mutable state explicitly as a `mut` parameter, or nest the helper `fn` inside the scope that owns the `var`. Top-level `let` values are readable everywhere.",
        "",
        "var acc = 0\nfn add(mut acc: Int, x: Int):\n    acc += x\nadd(mut acc, 1)\nprint(acc)",
        mode="reserved",
    ),
    # --- E05xx: Backends & Compatibility ---
    _entry(
        "E0501",
        "Unsupported construct in backend",
        "A construct passed syntax/type checking but is not valid in the execution/C++ backend.",
        "For example, `yield` is only valid inside a generator function. Restructure the construct or use the supported form.",
        "fn bad() -> Int:\n    yield 1\n    return 0\nprint(bad())",
        "fn ok() -> Int: 1\nprint(ok())",
        mode="build",
    ),
    _entry(
        "E0502",
        "Non-ASCII character in source or string",
        "CP Lang source files and `Str`/`Char` literals are pure ASCII.",
        "Replace smart quotes, dashes, or non-ASCII characters with plain ASCII equivalents.",
        'let s = "caf\u00e9"',
        'let s = "cafe"\nprint(s)',
    ),
    _entry(
        "E0503",
        "Removed syntax from an older version",
        "This spelling was removed in favour of a simpler, unified CP Lang form.",
        "Follow the diagnostic's help message to switch to the current spelling (for example `elif` instead of `else if`).",
        "if true:\n    print(1)\nelse if false:\n    print(2)",
        "if true:\n    print(1)\nelif false:\n    print(2)",
    ),
    _entry(
        "E0504",
        "Planned syntax, not implemented yet",
        "The CP Lang 0.3 design has this form, and this version does not implement it yet. The message names the design section that specifies it.",
        "Follow the help line to write it the way this version reads it; the rest of the program is still checked.",
        "let s: Str(len: 1..=10) = read()\nprint(s.len)",
        "let s: Str = read()\nprint(s.len)",
    ),
    # --- E06xx: Lends, Projections & Traits ---
    _entry(
        "E0601",
        "Aliased mutable lend",
        "The same variable was passed twice as `mut` (or mutably lent twice at once).",
        "CP Lang prevents mutable aliasing without lifetimes. Pass distinct places or copy one of the values first.",
        "fn add_both(mut a: Int, mut b: Int):\n    a += 1\n    b += 1\nvar x = 0\nadd_both(mut x, mut x)",
        "fn add_both(mut a: Int, mut b: Int):\n    a += 1\n    b += 1\nvar x = 0, y = 0\nadd_both(mut x, mut y)\nprint(x, y)",
    ),
    _entry(
        "E0602",
        "Variable mutated while lent",
        "A container was modified while a loop or slice was actively lending from it.",
        "Avoid mutating a collection while iterating over it (`for x in a:`) or holding a slice into it; collect updates separately or index by position.",
        "var a = [1, 2, 3]\nfor x in a:\n    a.push(x)",
        "var a = [1, 2, 3]\nlet n = a.len\nfor i in 0..n:\n    a.push(a[i])\nprint(a)",
    ),
    _entry(
        "E0603",
        "Lent value escapes its scope",
        "A lent slice or projection cannot be stored, assigned, or returned past its owner.",
        "Copy the slice into an owned `List` (for example `[x for x in a[l..r]]`) if you need it to outlive the borrow.",
        "fn bad(a: [0..3]Int) -> [0..2]Int:\n    return a[0..2]",
        "fn ok(a: [0..3]Int) -> List<Int>:\n    return [x for x in a[0..2]]\nprint(ok([1, 2, 3]))",
    ),
    _entry(
        "E0604",
        "Invalid `lends` projection",
        "A `lends` projection violated the projection rules (missing `lend`, wrong mutability, or invalid writeback).",
        "A projection method must end by lending a single sub-place of `self`, and any writeback after `lend` requires `mut self`.",
        "struct Box(v: Int)\nimpl Box:\n    fn view(self) lends Int:\n        let x = self.v",
        "struct Box(v: Int)\nimpl Box:\n    fn view(self) lends Int:\n        lend self.v\nlet b = Box(42)\nprint(b.view())",
    ),
    _entry(
        "E0605",
        "Invalid `Index` implementation",
        "An `impl Index` declaration used a non-constant `const K` or a mutable `var` for `let K`.",
        "Use `let K = ...` frozen from immutable values so array sizes cannot change after allocation.",
        "var n = 5\nstruct Id(i: Int)\nimpl Index for Id:\n    let K = n",
        "let n = 5\ndomain Id = 0..n\nprint(Id.count)",
    ),
    _entry(
        "E0606",
        "Missing trait implementation for operator",
        "An operator (`+`, `-`, `*`, etc.) was used on a struct that does not implement the corresponding trait.",
        "Implement the required operator trait (`Add`, `Sub`, `Mul`, ...) for the struct.",
        "struct Vec2(x: Int, y: Int)\nlet a = Vec2(1, 2) + Vec2(3, 4)",
        "struct Vec2(x: Int, y: Int)\nimpl Add for Vec2:\n    fn add(self, other: Vec2) -> Vec2: Vec2(self.x + other.x, self.y + other.y)\nlet a = Vec2(1, 2) + Vec2(3, 4)\nprint(a.x, a.y)",
    ),
    # --- W01xx: Warnings ---
    _entry(
        "W0101",
        "Unused expression value",
        "A standalone statement computed a value and discarded it.",
        "Assign the result to a binding, print it, or remove the no-op expression.",
        "let x = 1\nx + 2\nprint(x)",
        "let x = 1\nlet y = x + 2\nprint(y)",
    ),
    # --- R00xx: Runtime traps ---
    _entry(
        "R0001",
        "Runtime trap",
        "The program trapped at run time (out-of-bounds index, overflow, division by zero, failed `assert`, or `!` on `none`).",
        "Interpreter and sanitized C++17 trap identically with the source location and local values. Fix the invariant or guard the operation before unwrapping or indexing.",
        "let x: ?Int = none\nprint(x!)",
        "let x: ?Int = none\nprint(x ?? -1)",
        mode="run",
    ),
    # --- L0xxx: Lints ---
    _entry(
        "L0001",
        "Unknown lint name",
        "`@allow(...)` or `cp.toml` names a lint that does not exist.",
        "Check the lint name against the Error Index (for example `var_never_reassigned`, `domain_keyed_map`, `float_equality`).",
        "@allow(no_such_lint)\nfn f() -> Int: 1\nprint(f())",
        "fn f() -> Int: 1\nprint(f())",
        mode="lint",
        lint_name="unknown_lint",
    ),
    _entry(
        "L0101",
        "Position arithmetic via `.ord`",
        "`V(v.ord + 1)` constructs a domain position through raw integer arithmetic.",
        "Use `v + 1` (returning `?V`) or `v.cyc(1)` instead of round-tripping through `.ord`, so out-of-bounds steps cannot happen silently.",
        "domain V = 1..=5\nlet v = V.first\nlet next = V(v.ord + 1)\nprint(next)",
        "domain V = 1..=5\nlet v = V.first\nif let next = v + 1:\n    print(next)",
        mode="lint",
        lint_name="position_arithmetic",
    ),
    _entry(
        "L0201",
        "`var` is never mutated",
        "A binding was declared `var` but never reassigned or modified.",
        "Use `let` so readers and the compiler know the value stays constant.",
        "var x = 10\nprint(x)",
        "let x = 10\nprint(x)",
        mode="lint",
        lint_name="var_never_reassigned",
    ),
    _entry(
        "L0202",
        "Struct field is never read",
        "A struct declares a field that is never read anywhere in the file.",
        "Remove unused fields to keep structs compact and cache-friendly, or read the field where intended.",
        "struct Item(w: Int, unused: Int)\nlet it = Item(3, 9)\nprint(it.w)",
        "struct Item(w: Int)\nlet it = Item(3)\nprint(it.w)",
        mode="lint",
        lint_name="unread_field",
    ),
    _entry(
        "L0203",
        "Local name shadows a top-level `let`",
        "A local binding hides a top-level `let` constant or input size.",
        "Shadowing top-level `n`, `m`, or `MOD` inside a helper is a frequent contest bug. Pick a distinct local name.",
        "let n: 1..=9 = read()\nfn f(k: Int) -> Int:\n    let n = k + 1\n    return n\nprint(f(n))",
        "let n: 1..=9 = read()\nfn f(k: Int) -> Int:\n    let next = k + 1\n    return next\nprint(f(n))",
        mode="lint",
        lint_name="shadows_top_level",
    ),
    _entry(
        "L0301",
        "Exact equality on `F64`",
        "`==` or `!=` was used on floating-point values.",
        "Floating-point rounding makes exact equality brittle (outside exact `== 0.0`). Compare with a tolerance such as `abs(a - b) < 1e-9`.",
        "let a = 0.1 + 0.2\nlet b = 0.3\nif a == b:\n    print(1)",
        "let a = 0.1 + 0.2\nlet b = 0.3\nif abs(a - b) < 1e-9:\n    print(1)",
        mode="lint",
        lint_name="float_equality",
    ),
    _entry(
        "L0412",
        "`Map` or `Set` keyed by a domain",
        "A `Map<V, T>` or `Set<V>` is keyed by a dense domain `V`.",
        "An array `[V]?T` or `[V]Bool` uses one flat slot per element with O(1) zero-hash access and no allocation overhead.",
        "domain V = 1..=5\nvar m: Map<V, Int> = {}\nm[V.first] = 1\nprint(m.len)",
        "domain V = 1..=5\nvar m: [V]?Int = none\nm[V.first] = 1\nprint(m[V.first] ?? 0)",
        mode="lint",
        lint_name="domain_keyed_map",
    ),
    _entry(
        "L0413",
        "`@memo` table exceeds memory budget",
        "A `@memo` function's parameter domains multiply to a table too large for contest memory limits.",
        "Reduce the state dimensions, roll one dimension in an iterative DP, or switch to a sparse state representation.",
        "domain A = 0..100_000\ndomain B = 0..100_000\n@memo\nfn dp(a: A, b: B) -> Int: a.to_idx + b.to_idx\nprint(dp(A.first, B.first))",
        "domain A = 0..100\ndomain B = 0..100\n@memo\nfn dp(a: A, b: B) -> Int: a.to_idx + b.to_idx\nprint(dp(A.first, B.first))",
        mode="lint",
        lint_name="memo_key_space",
    ),
    _entry(
        "L0414",
        "Checked `List` indexing inside nested loops",
        "A `List` is indexed inside nested loops where the compiler cannot prove bounds statically.",
        "Use a fixed domain array `[D]T` or iterate directly (`for x in a:`) to eliminate inner-loop bounds checks.",
        "let a = [1, 2, 3, 4]\nvar s = 0\nfor i in 0..4:\n    for j in 0..4:\n        s += a[j]\nprint(s)",
        "domain I = 0..4\nlet a: [I]Int = [1, 2, 3, 4]\nvar s = 0\nfor i in I:\n    for j in I:\n        s += a[j]\nprint(s)",
        mode="lint",
        lint_name="unproven_list_index",
    ),
    # --- E9999: Internal ---
    _entry(
        "E9999",
        "Internal compiler error",
        "The compiler hit an unexpected internal invariant failure.",
        "This code is never caused by a user mistake. Please file a minimal reproducer so the compiler bug can be fixed with a regression test.",
        "",
        "",
        mode="internal",
    ),
]


def discover_rust_codes(repo_root: Path) -> list[str]:
    """Extract every diagnostic and lint code defined in `cpl-diag` and `cpl-lint`."""
    found: list[str] = []
    seen: set[str] = set()
    for rel in ("crates/cpl-diag/src/lib.rs", "crates/cpl-lint/src/lib.rs"):
        text = (repo_root / rel).read_text()
        for code in re.findall(r'Code\("([A-Z]\d{4})"\)', text):
            if code not in seen:
                seen.add(code)
                found.append(code)
    return found


def category_for(code: str) -> str:
    for label, prefix in CATEGORY_ORDER:
        if code.startswith(prefix):
            return label
    return "Other"


def error_slug(code: str) -> str:
    return f"error-{code}"


def render_error_index_html() -> str:
    """Render the body HTML for `errors.html`."""
    grouped: dict[str, list[dict]] = {label: [] for label, _ in CATEGORY_ORDER}
    for entry in ERROR_CATALOG:
        grouped.setdefault(category_for(entry["code"]), []).append(entry)

    parts = [
        "<p>Every diagnostic, warning, runtime trap, and lint code emitted by "
        "<code>cpl check</code>, <code>cpl run</code>, <code>cpl build</code>, "
        "<code>cpl lint</code>, and <code>cpl lsp</code> has a dedicated page below. "
        "Every trigger and fix snippet is verified automatically in <code>scripts/check.sh --fast</code>.</p>",
    ]
    for label, _ in CATEGORY_ORDER:
        entries = grouped.get(label) or []
        if not entries:
            continue
        slug = re.sub(r"[^a-z0-9]+", "-", label.lower()).strip("-")
        parts.append(f'<h2 id="{slug}">{html.escape(label)}</h2>')
        parts.append(
            "<table><thead><tr><th>Code</th><th>Title</th><th>Summary</th></tr></thead><tbody>"
        )
        for entry in entries:
            code = html.escape(entry["code"])
            href = f'{error_slug(entry["code"])}.html'
            title = html.escape(entry["title"])
            summary = html.escape(entry["summary"])
            lint_tag = (
                f' <code>{html.escape(entry["lint_name"])}</code>'
                if entry.get("lint_name")
                else ""
            )
            parts.append(
                f'<tr><td><a href="{href}"><code>{code}</code></a></td>'
                f'<td><a href="{href}">{title}</a>{lint_tag}</td>'
                f"<td>{summary}</td></tr>"
            )
        parts.append("</tbody></table>")
    return "\n".join(parts)


def render_error_page_html(entry: dict, highlight_cp) -> str:
    """Render the body HTML for an individual `error-<CODE>.html` page."""
    code = html.escape(entry["code"])
    category = html.escape(category_for(entry["code"]))
    parts = [
        f'<p><a href="errors.html">&larr; Back to Error Index</a> &middot; <strong>{category}</strong></p>',
        f'<p>{html.escape(entry["summary"])}</p>',
        f'<h2 id="why">Why this happens</h2>',
        f'<p>{html.escape(entry["explanation"])}</p>',
    ]
    if entry.get("lint_name"):
        name = html.escape(entry["lint_name"])
        parts.append(
            f'<p>Suppress on a single item with <code>@allow({name})</code> or configure '
            f'<code>{name} = "allow"</code> under <code>[lints]</code> in <code>cp.toml</code>.</p>'
        )
    if entry["trigger"]:
        label = (
            "Trigger (triggers lint)"
            if entry["mode"] == "lint"
            else "Trigger (traps at run time)"
            if entry["mode"] == "run"
            else "Trigger (rejected by compiler)"
        )
        parts.append(f'<h2 id="trigger">{label}</h2>')
        parts.append(
            '<div class="code-block">'
            f'<div class="code-bar"><span>{code} trigger</span>'
            '<button class="copy" type="button" data-copy>Copy</button></div>'
            f'<pre><code class="language-cp">{highlight_cp(entry["trigger"])}</code></pre>'
            "</div>"
        )
    if entry["fix"]:
        parts.append('<h2 id="fix">Fix</h2>')
        parts.append(
            '<div class="code-block">'
            f'<div class="code-bar"><span>{code} fix</span>'
            '<button class="copy" type="button" data-copy>Copy</button></div>'
            f'<pre><code class="language-cp">{highlight_cp(entry["fix"])}</code></pre>'
            "</div>"
        )
    if entry["mode"] == "internal":
        parts.append('<h2 id="reporting">Reporting an internal error</h2>')
        parts.append(
            "<p>Run <code>cpl check</code> on the smallest file that reproduces the failure "
            "and open an issue with the file and compiler version.</p>"
        )
    return "\n".join(parts)
