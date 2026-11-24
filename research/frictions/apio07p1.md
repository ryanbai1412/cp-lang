# apio07p1

Ported from `corpus/submissions/5532655.cpp`. No `dream.cp`; the dream is below.

## Dream
`port.cp` as written. `kids: [Bar](l: ?Bar, r: ?Bar) = read(none: -1)` turns the statement's -1
into an absent child, and `span(u, d) -> ?Span` returns either the subtree's (deepest,
shallowest, swaps) or `None` if it cannot be sorted. `let ... else: return None` stops at the
first unsortable subtree. The accepted C++ does the same with a global flag and depth arrays.

## No gaps
Every object in the editorial (a subtree's depth span, keep or swap, the root's span <= 1) is a
named value. The -1 sentinel is gone, and failure is `?Span` instead of a flag.

## Measurements
- Tokens: port 242, original 471.
- `--max` (n = 1e5, median of 15): port 23.7 ms / 11.9 MB, original 23.3 ms / 6.5 MB (1.02x).
  Same output. The extra 5 MB, inside the 32 MB limit, is likely the `span` recursion
  down a deep tree (not profiled by allocation). Time is at par, so no friction is filed.
- Emitted C++: 14,270 bytes.
