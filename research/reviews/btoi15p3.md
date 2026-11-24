# btoi15p3

Score: 5/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header proves the ceil(L / 2)
lower bound and why pairing leaf i with leaf i + L/2 in preorder covers every tree edge. The
code is that proof: root at an inner vertex with `V.find`, preorder leaves from std `Rooted`,
and one print loop.

## Checklist failures
None found.

## Owed rewrites
None.

## Language gaps
- The corpus suite compares tokens instead of running `checker.py`, so other optimal link sets
  read `wrong`: in the friction (spec).

## Dream notes
The body is the port's. Its two-line header drops the lower bound and the block argument that
make `(leaves.len + 1) / 2` and `i + leaves.len / 2` checkable; the port's header has them.
