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
  are always present, so the typed view's accessors return nodes, not
  options, as today's AST does.
- **Trivia placement:** a node starts at its first token and ends at
  its last; trivia between tokens sits in the innermost node that holds
  both neighbours. So a node's range is today's span.
- **File:** `File` holds one `Item` node per top-level item (today's
  `split_items` rule, round 5's column-0 keyword rule), trivia between
  items, and the final `Eof`.

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
children by kind and position. Desugarings today's parser does
(`a is P and b`, `(lo..hi) by s`, `let a = x and let b = y`, the
`??=` lead statement, unpacked parameters) are not in the tree: the
tree says what was written, and the reader that gives it meaning
desugars, once, as a fn of the typed node.

## Error recovery

Kept from today's parser, construct by construct: one error per
statement, then resync at the line's end; a broken header's block is
still parsed; brackets the lexer closed stay closed. Skipped tokens go
in an `Error` node, an absent child is `Missing`. Each item is parsed
alone, so an error never moves another item's boundaries.

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
   the same landing. Test: the lowering equals today's parse (trees and
   diagnostics) on every file before the switch; after it,
   `cpl-syntax`'s own parser tests keep running through the lowering.
   With injected errors every unaffected item still parses the same.
3. The typed view, item ids and node indexes.
4. Readers move one at a time, each deleting its `cpl_syntax` use:
   `cpl-next-decls` (modules, items, signatures), then
   `cpl-next-check`, `-lower`, `-link`, `cpl-next`.
   `next_syntax_old_refs` in `scripts/debt-ratchet.py` counts the
   cpl-next source lines naming `cpl_syntax`; it only shrinks, to 0.
5. At stage 8 the lints, formatter and editor read the tree, and the
   lowering and `cpl-syntax` are deleted with the old pipeline.

## Calls

- `cpl-next-syntax` depends on `cpl-diag` only, for spans and the one
  diagnostic code table; it is not a compiler crate.
- Trivia are sibling tokens, not leading and trailing lists on a token
  (contract 1's `Token` shape): each byte is in exactly one token, and
  a token's leading trivia are its preceding trivia siblings.
- No `rowan`: the tree is small, and a new third-party crate is a
  reviewed decision for little gain.
