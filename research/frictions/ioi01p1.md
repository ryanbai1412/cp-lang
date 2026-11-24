# ioi01p1: Mobile Phones

The dream reads the commands as a tagged enum; the port reads an `Int` op
code and branches on it.

## Enum variants with io tags
- **Where:** port lines 16-24: `let op: Int = read()` then
  `if op == 1 ... elif op == 2 ... else: break`. The dream's
  `enum Cmd: Size(s: Int) = 0 | Add(x: Int, y: Int, a: Int) = 1 | ...`
  is E0107 "expected the end of the statement, found `=`" today.
- **What felt wrong:** the op codes are magic numbers, and "3 ends" is the
  `else` of a chain.
- **Goal it hurts:** express, correct (an unknown op silently ends input).
- **Ideal code:** `for cmd in read stream Cmd until End: match cmd: ...`.
- **General fix:** io tags on enum variants (`= 0`, `= 'I'`) so `read<Cmd>()`
  dispatches on the tag; the same gap blocks ioi05p2's dream.
- **Severity:** medium (every command-stream problem).
