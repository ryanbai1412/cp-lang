# Contract 1: snapshot → syntax tree

One lexer and one parser. Input: one file's text. No other layer reads
source text.

## What crosses

```rust
struct SyntaxFile {
    root:    Node,                    // lossless: printing it gives back every byte
    items:   List<ItemSyntax>,        // top-level declarations and statements, in order
    errors:  List<SyntaxError>,
    replaced: List<(UnitId, UnitId)>, // "this statement replaced that one", against the previous snapshot
}
struct ItemSyntax {
    id:      UnitId,                  // see facts/identity.md
    kind:    Decl | Statement,
    tokens:  TokenHash,               // tokens without whitespace and comments
    node:    Node,
    decorators: List<Node>,           // attached here, never a separate item
}
enum NodeKind { /* one per grammar production */, TypeOrExpr, Error, Missing }
struct SyntaxError { code, at: Node, message, fixes: List<Fix> }   // every rejected spelling carries its fix
struct Token { kind, text_range, leading_trivia, trailing_trivia }
```

## Producer guarantees

- **Total.** Any byte sequence gives a tree. Bad input becomes `Error`
  and `Missing` nodes plus entries in `errors`.
- **Lossless.** Comments and whitespace are in the tree.
- **Local.** An edit inside one item re-parses that item only. A
  top-level item starts at a line in column 0 outside brackets, except
  that an `else` or `elif` line and the line after a decorator belong
  to the item they attach to. Deeper lines, continuation lines
  included, belong to the item above. There are no multi-line strings
  or block comments, so this needs no parse.
- **Stable names.** An item's `id` does not change when lines are
  added above it, when it is reformatted, or when a comment changes.
- The grammar is one data file. The parser, the editor grammars, the
  formatter's node list and the spec's grammar chapter are generated
  from it or tested against it.

## Consumer may assume

- Every `Node` has a kind from the closed list and a text range.
- Node indexes inside an item depend only on that item's tokens.

## Must never

- No consumer may re-lex text, match on token text to find structure,
  or store a text range in anything that outlives the snapshot. Store
  `(UnitId, node index)` and ask for the range.

## Checked by

- Fuzz: random bytes never crash and always print back exactly.
- Property: re-parsing after any single-item edit equals a full parse.
- Scan: no use of the lexer outside this layer and the formatter.

## Open

- *Closed by reading the grammar chapter:* no statement changes how
  later lines parse. Case decides type against value, contextual words
  are positional, and a type argument that may be an expression is one
  `TypeOrExpr` node the checker settles. Parsing needs no name lookup.
- Identity for two items with the same tokens when one is edited: the
  counter among equals shifts. Accepted for now.
- **Audit 4, finding 10.** `tokens` must include INDENT, DEDENT and
  NL: two programs with the same other tokens print 13 and 11.

## Round 5 (2026-11-25, not yet reviewed)

- **Item boundaries.** A line at column 0 can sit inside brackets, a
  string or a body comment, so "column 0 starts an item" is wrong.
  Rule: a column-0 line that starts with a statement keyword always
  starts an item, as error recovery, so an unclosed bracket in item k
  cannot move the boundaries and identities of later items.
- **Today's parser is not this contract.** It is a hand-written tree
  with spans, with no trivia in the tree and no grammar data file. A
  lossless tree is a rewrite of that crate and is its own migration
  stage, after the tools are on the facade.
