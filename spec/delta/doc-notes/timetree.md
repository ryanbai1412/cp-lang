# timetree doc notes

- `TimeTree.add_interval`: the spec gives no bounds rule; chose: an empty interval schedules nothing, a non-empty one reaching outside `0..q` traps (matches range indexing elsewhere).
- `TimeTree.event`: the spec does not say what an `Apply` id is; chose dense ids `0, 1, ...` in the order node copies are stored, with programs told to read them from the steps.
- `TimeTree.steps`: chose children left (earlier time) before right and applies in scheduling order within a node, undos LIFO; the spec says only "DFS".
- `TimeTree.run`: "each non-empty node" read as a node holding at least one event; nodes without events call neither `mark` nor `undo`.
- `cdq`: the stub says the halves are solved before `on_cross`, which is the order for independent halves; the classic online CDQ (left, cross, right) is not expressible with this signature. Left as stated; the left half is the first `n / 2` events (chosen).
- `offline_over`: chose list order for scheduling items, so applies within a node follow item order.
