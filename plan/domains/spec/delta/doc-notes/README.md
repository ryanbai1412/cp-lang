# Library doc notes

One file per `std` module, written while bringing it to the doc
standard (`std/README.md`). Each bullet is either a **choice** (the
spec was silent, so the docs pin a behaviour, with the reason) or a
**contradiction** (the stub and `spec/lib`, `spec/core` or `decisions.md`
disagree; the docs follow the stub until it is resolved).

Resolving a contradiction: decide which side is right, edit the losing side
(stub, library chapter or decision), and mark the bullet `(resolved)` with
the outcome. A choice stands unless a later decision overrides it.
