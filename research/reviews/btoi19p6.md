# btoi19p6

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header
explains Lawler's k-best partitioning: one child per free slot, keeping the slots before it and
banning its member. `best` fills free slots greedily. The heap, though, holds indices into an
arena.

## Checklist failures
- idiomatic-code/R2: `subs: List<Sub>` plus `MaxHeap<(Int, Int)>` of (value, index) (lines
  38-43, 51) is an arena the friction does not justify. Its `Heap(by:)` failure is for a
  `[P]Bool` or `Set<P>` field, and the port already uses `List<Bool>`, so a heap of `Sub` by
  `value` should work today.
- idiomatic-code/R6: `banned[p.ord]` (lines 31, 48); in the friction, with the E0309 repro.

## Owed rewrites
- Push `Sub`s into a heap ordered by `value` and split the popped `Sub`, dropping `subs`/`at`.
- Add a run to the friction for the proposed `best_first` (idiomatic-code/R11).

## Language gaps
- `Heap(by:)` rejecting a struct with a domain-array field: in the friction, with a repro.
- No best-first enumeration combinator: in the friction.

## Dream notes
`@get fn value(self) -> Int = Ev.sum(e => self.team.max(by: p => score[p, e]))` sums the
best members, not their scores; it should be `score[self.team.max(by: ...)!, e]` or the port's
`team.map(p => score[p, e]).max()!` (caps at 3, a wrong fact).
