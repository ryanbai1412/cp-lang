# btoi17p6

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp`, without the C++. The bottom-up greedy is one
`tree.up` with a `Pick` monoid. `join` keeps both shallowest marks when they are >= d apart and
otherwise drops the shallower one, which is the editorial in a few lines.

## Checklist failures
- idiomatic-code/R1: the monoid identity `(count: 0, near: 2 * d)` (line 31) is a "no mark"
  sentinel that the header never explains. It works because `2 * d + anything >= d` always
  keeps the other side, but that is not written down.
- idiomatic-code/R10: the port takes 0.05 s against the C++'s 0.02 s (2.5x, the friction's
  measurement) with no performance friction. Filed in the friction now.

## Owed rewrites
- Say why `2 * d` is a safe identity, or make the summary `?Pick` with `None` as the identity.

## Language gaps
- 2.5x at `--max`: added to the friction (perf).

## Dream notes
The dream keeps the same unexplained `2 * d` (caps at 3). It is also worse than the port on
R6: `Int(v) == 0` and `parent[Int(v)]` cross from `V` to `Int`, where the port uses
`Kid.at(v.ord)`.
