# External Parameter Labels (Option A — Proposal)

*Status: proposed, pending Ryan's decision. Do not implement until accepted.*

## Motivation

Built-in catalog rows already use keyword-shaped call-site labels such as
`xs.reach(while: p)`, and stdlib containers want `s.drain(after: x, while: p)`.
Today, a call site accepts any keyword before `:` (`Parser::arg`), but a
function declaration cannot declare a `while:` parameter because `while` is a
keyword in parameter position and cannot be referenced inside the body.

Leaving `reach(while:)` in the built-in catalog while forbidding stdlib and
user functions from writing the same signature is stdlib privilege, which
`AGENTS.md` forbids.

## Surface Syntax

A parameter may carry an explicit external call-site label before its internal
binding name:

```cp
fn drain(mut self, after: K, while keep: (K) -> Bool) -> List<K>:
    // Callers write: s.drain(after: x, while: k => k <= hi)
    // Inside the body, the parameter is bound as `keep`.
    ...
```

General form:

```text
param := [ 'mut' | 'var' ] [ label ] ident ':' type [ '=' expr ]
label := ident | keyword   // only when followed by `ident ':'`
```

Rules:
1. When `label` is omitted, the external label and internal binding name are
   identical (`after: K`), preserving 100% of existing declarations.
2. When `label` is present (`while keep: (K) -> Bool`), callers pass the
   argument by `label:` (`while: p`) or positionally, and the body binds
   `keep`.
3. `label` may be a keyword (`while`, `in`, `for`, `if`, `from`, `to`, `by`,
   ...) or an ordinary identifier (for readable call sites like `at index: Pos`
   or `from start: V`).
4. Two parameters on the same function cannot share the same external label;
   an internal binding name still obeys ordinary local-scope uniqueness.
5. `label == ident` (e.g. `keep keep: T`) is redundant and flagged by a lint
   suggesting `keep: T`.

## Compiler & Tooling Impact

### 1. AST & Parser (`cpl-syntax`)
- Extend `ast::Param` with `pub label: Option<Name>` (or `pub external: Name`
  defaulting to `name`, plus `label_span: Option<Span>`).
- In `Parser::param`, after optional `mut`/`var`: if the current token is an
  identifier or keyword AND `peek_n(1) == T::Ident` AND `peek_n(2) == T::Colon`,
  consume the external label as `label` and the identifier as `name`.
- No ambiguity with existing syntax: `mut x: T` and `var x: T` are parsed as
  modifiers first (so `mut` and `var` remain modifiers unless followed by
  `ident ident ':'`, or simply disallow `mut`/`var` as bare external labels
  without a preceding modifier — better: `mut`/`var` are always parameter
  modifiers, never external labels).
- Call sites (`Parser::arg`) already accept keywords before `:` (`while: p`);
  no call-site grammar change is needed.

### 2. Checker (`cpl-check`)
- `def::Param` gains `pub label: String` (defaulting to `name`).
- Named-argument matching in `check/call.rs` matches `arg.name` against
  `param.label` instead of `param.name`.
- Error messages (`missing argument`, `unknown named argument`, `takes T for
  <label>`) display `param.label` at call sites, matching what callers write.
- Function body scope binds `param.name` (`keep`), never `param.label`.
- Duplicate external label check on `fn` definitions emits `E0202` on the
  second label.
- Trait impl conformance checks parameter labels alongside types when a trait
  method declares a label.

### 3. IR, Interpreter, Codegen (`cpl-ir`, `cpl-interp`, `cpl-codegen`)
- `cpl-ir/src/lower/call.rs` matches named arguments by `param.label` and
  binds the lowered parameter slot to `param.name`.
- No IR instruction, VM opcode, or C++ codegen representation changes: by the
  time IR is produced, all calls are positional and all locals use `param.name`.

### 4. Formatter (`cpl-fmt`)
- Print `label name: Type` with a single space between `label` and `name` when
  `param.label.is_some()` and `label != name`.
- `formats_the_whole_corpus` AST-equivalence test covers round-tripping
  automatically.

### 5. LSP (`cpl-lsp`)
- Hover, signature help, and named-argument completion display `label name: T`
  in signatures and insert `label: ` at call sites.
- Go-to-definition on a call-site `label:` jumps to the parameter's `label`
  span; rename on `name` inside the body does not break call-site `label:`
  (and vice versa).

### 6. Docs & Codemod
- Document in `site/content/functions.md` (or `types-and-control.md`) alongside
  named arguments, plus one example in `decisions.md`.
- **Codemod:** none needed. The feature is strictly additive and backwards
  compatible with all existing code, stdlib, and `xs.reach(while:)` calls.
- Once landed, `stdlib/seq.cp` / `stdlib/containers.cp` can declare
  `while keep:` directly, eliminating the built-in catalog privilege for
  `reach(while:)`.
