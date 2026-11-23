// A type's own fn, from an impl or its body, is what `x.f()` calls, even
// beside a top-level `fn f(self: S)` whose bound the type lacks.
use std.algebra.{Reversible}
struct Word(s: Str):
    fn shout(self) -> Str = s + "?"
impl Reversible for Word:
    fn rev(self) -> Word = Word(s + "!")
let w = Word("ab")
print(w.rev().s, w.shout(), [1, 2, 3].rev.first!)
