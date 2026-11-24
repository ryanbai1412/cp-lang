// expect: E0203
// has: "empty when its `len` is 0"
// On a type spelled with `len` and no `is_empty`, the unknown-member error for `is_empty` offers `len == 0` (bug-catch round 14, 28ca9a32). Std's sized types declare `is_empty` (containers.md), so the case uses a struct of its own.
struct Box(len: Int)
let b = Box(0)
if not b.is_empty: print(1) // <- here
