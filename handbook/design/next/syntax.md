# The lossless syntax tree

**Status:** 2026-11-27, lane J. Design for contract 1
(`contracts/01-syntax.md`) and migration stage 8's first half: one
lexer and one parser, in `crates/cpl-next-syntax`, under every reader.

## Summary

```text
text ─▶ lex (every byte a token) ─▶ split items ─▶ parse each item ─▶ green tree ─▶ red tree ─▶ typed view
                                                                                     │
                                       cpl-syntax's AST (old pipeline, tools) ◀──────┘ lowering, until stage 8
```

One parser exists in the repository from the switching landing on.
`cpl-syntax` keeps its AST types, and its `parse_file` becomes a
lowering of the new tree, so the old pipeline, the tools and the not
yet migrated cpl-next readers all read the one parser. No old parser
survives beside the new one (migration rules 5 and 10).

## Tree shape

- **Tokens.** The lexer is today's layout lexer, kept whole, plus
  trivia: every byte that is not a token (spaces, `\n`, `//` comments,
  a skipped foreign header) is a trivia token. Layout tokens
  (`Newline`, `Indent`, `Dedent`, `Eof`) and the closers the lexer
  inserts for an unclosed bracket are zero-width. So the concatenated
  token texts are the file, byte for byte.
- **Green tree** (rust-analyzer's design, written here in about 300
  lines, so no new crate): immutable `Arc` nodes of `(kind, width,
  children)`, tokens of `(kind, text)`. Positions are not stored, so
  an unchanged item's green node is reused as is.
- **Red tree:** a cursor `(green, parent, offset)` made on demand.
  `Send + Sync`, so declarations may keep nodes.
- **Kinds:** one closed `SyntaxKind`, token kinds and node kinds,
  including `Error` (bytes the parser skipped) and `Missing` (a
  zero-width node where a required child is absent). Required children
  are always in the tree, as `Missing` when not written.
- **Trivia placement:** a node starts at its first token and ends at
  its last; trivia between tokens sits in the innermost node that holds
  both neighbours. So a node's range is today's span.
- **File:** `File` holds one `Item` node per top-level item (today's
  `split_items` rule, round 5's column-0 keyword rule), trivia between
  items, and the final `Eof`. Each item is parsed on its own, over its
  own tokens, so an edit to one item re-parses only that item.
- **Names:** a name a declaration introduces (a fn's, type's, field's,
  variant's, law's, module's) sits in a `Name` node, so readers find it by
  kind and a missing name is a `Name` holding `Missing`.
- **What an item holds:** its decorators, and the comment lines directly
  above it: contiguous `//` lines with no blank line between them or below
  the last. So docs and hover read them from the item, and moving or
  formatting an item carries them. A comment after a blank line stays in
  `File`.

## The grammar is data

- `src/ops.rs` is the operator table: spelling, token, binary or
  unary, precedence level, associativity. The lexer's punctuation, the
  parser's precedence climbing and the spec's precedence list (§1.5,
  checked by a test) read it.
- `src/grammar.rs` is one table of node kinds and their typed
  accessors; a declarative macro turns it into `SyntaxKind` and the
  typed view, so neither is written twice.

## Typed view

rust-analyzer style: a typed node is a newtype over a red node of one
kind (`ast::FnDecl`, `ast::BinExpr`), and an enum per category
(`ast::Expr`, `ast::Stmt`, `ast::Type`, `ast::Pattern`). Accessors find
children by kind and position, and are type-honest: an accessor for a
child returns `Option<ast::X>`, `None` where the child is `Missing` or an
`Error` node, so no reader can take a parse error for a valid node; the
parse error already has its diagnostic, so the reader stays quiet. Desugarings today's parser does
(`a is P and b`, `(lo..hi) by s`, `let a = x and let b = y`, the
`??=` lead statement, unpacked parameters) are not in the tree: the tree says what was written. Every desugaring lives in one module,
`desugar`, one fn per sugar over the typed node, which every reader calls;
check and lower never desugar on their own.

`src/ast.rs` holds the views: one per node kind, and the categories
`Decl`, `Stmt`, `Type`, `Expr`, `Pattern` and `GenericArg`. Its
accessors so far are what ids and decls read: an item's decorators, `pub`
and name; a fn's generics, params (`mut`, `var`, label, named-only,
variadic, default), return (`yields`, `lends`), `uses`, where clause,
body and `signature_span`; struct, enum, trait and impl members, fields,
variants, aliases, consts, domains, contexts and `use`; every type; and
the expressions decls reads (names, literals, fields, parens, binary,
prefix, ranges, calls, generic arguments). Check's accessors come with
check's move. Decorators before a member are inside its node. A mod's
items are `Item` nodes, read like top-level ones. An impl, an operator fn
and a statement have no name. A `const` is a `ConstDecl` wherever it
appears. A test checks that every child a view reads in std and
conformance has a view.

## Error recovery

Designed for the tree, not copied from today's one error per statement
and resync at the line's end. Each construct recovers inside itself
with a recovery set, rust-analyzer style: the tokens that may follow it
or close an enclosing construct.

- **Expression:** a token that cannot start an operand becomes an
  `Error` node holding it, or a zero-width `Missing` operand when it is
  in the enclosing construct's set; the operator and the other operand
  stay in the tree.
- **Argument and bracket lists:** a broken element ends at the next `,`
  or the closer; the elements after it parse as usual.
- **Block:** a broken statement ends at its `Newline`; the block's
  other statements, and a broken header's block, still parse.
- **Pattern:** a broken pattern ends at `=>`, `:`, `,`, `|` or the
  closer, so the arm's body still parses.
- Brackets the lexer closed stay closed; an item is parsed alone, so an
  error never moves another item's boundaries.

One diagnostic per `Error` or `Missing`, at the first bad token, never
a cascade. So a broken sub-expression leaves the rest of its statement
in the tree, for completion and hover. Tests inject errors and check
the surrounding nodes are kept, not only the other items.

## Ids and spans

- An item's `ItemId` stays today's path (module, name, nth);
  `tokens`, its `TokenHash`, hashes the non-trivia tokens, layout
  included (audit 4, finding 10).
- A node inside an item is `(ItemId, index)`, its preorder index in
  the item's subtree, which depends only on the item's tokens.
- Spans are item-relative: a node's range minus its item's start. The
  item's start is the `spans` query's only input, so a blank line above
  an item changes nothing else.

## Migration order

Each step lands on its own, with its test, and deletes what it
replaces.

1. The crate: kinds, green/red tree, lossless lexer. `cpl-syntax`'s
   lexer becomes the new one with trivia dropped. Test: every
   conformance, std and test-program file round-trips byte for byte.
2. The parser, building the tree; `cpl-syntax::parse_file` and
   `parse_item` become its lowering, and today's parser is deleted in
   the same landing. This edits frozen `cpl-syntax`: allowed because
   it deletes the old parser and cuts lines, and the commit body says
   so. Tests: the lowered AST equals today's parse on every file. Each
   diagnostic that differs from today's is a row of a reviewed list
   (case, old, new, why the new one is better), which can only shrink;
   old quirks are not reproduced to empty it. With injected errors
   every unaffected item, and the nodes around each error, still parse.
3. The typed view, item ids and node indexes.
4. Readers move one at a time, each deleting its `cpl_syntax` use:
   `cpl-next-decls` (modules, items, signatures), then
   `cpl-next-check`, `-lower`, `-link`, `cpl-next`.
   `next_syntax_old_refs` in `scripts/debt-ratchet.py` counts the
   cpl-next source lines naming `cpl_syntax`; it only shrinks, to 0.
5. At stage 8 the lints, formatter and editor read the tree, and the
   lowering and `cpl-syntax` are deleted with the old pipeline.

Old code reused, and where it goes (migration rule 13):

| Reused | Deleted in | Counter at 0 there |
|---|---|---|
| `cpl-syntax`'s parser, until the tree has one | step 2 | `old_parser_lines` |
| The lexer's trivia-free token list and `comments`, which the old parser reads and `Lexed::lossless` refills | step 2: the lexer emits trivia itself | `old_parser_lines` |
| `cpl-syntax`'s AST, as the lowering's output | stage 8, with the old pipeline | `next_syntax_old_refs` reaches 0 at step 4 |

The layout lexer itself is not reused old code: indentation is the
language, and the new crate owns it.

## Calls

- `cpl-next-syntax` depends on `cpl-diag` only, for spans and the one
  diagnostic code table; it is not a compiler crate. Nor is it a path to
  switch, so `cpl-syntax` may depend on it (`tests/unshipped.rs` exempts
  it). Its code counts like any `cpl-next` crate's: every keyword and
  foreign keyword is spelled once, as a row of `grammar.rs`'s tables, and
  the lexer looks words up there, so no code compares text with a fixed
  word.
- Parse speed is an end-state number (`plan/end-state.toml`): the whole
  conformance suite's parse time, today's parser against the tree.
- Trivia are sibling tokens, not leading and trailing lists on a token
  (contract 1's `Token` shape): each byte is in exactly one token, and
  a token's leading trivia are its preceding trivia siblings.
- No `rowan`: the tree is small, and a new third-party crate is a
  reviewed decision for little gain.

**`<` after a name.** `Name<..>` in an expression is type arguments when the
`<..>` closes on the line and what follows can follow a value; a lowercase
name takes them only before a call and only when no argument is a bare
lowercase name, so `a < b > (c)` stays a comparison. No other expression
form looks ahead past its own brackets except a table, `[D] d => body`.
