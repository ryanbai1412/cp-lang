# noi06p1

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`, a fragment). The header argument is clear (a flag per router, each leaf charged alone through a 2-D `Prefix`), and `best` follows it: a leaf's two costs, then a router's two flags combining its children's tables (lines 34-56).

## Checklist failures

- idiomatic-code/R6: heap nodes are raw `Int` with shifts (`2 * u`, `u >> (m - d - 1) ^ 1`, lines 28-31, 42, 52) and the flags are an `Int` bitmask (`flags.bit(d)`, `Int(more_b) << depth`, lines 43, 51). Both reported.
- The flow triangle is read with a hand double loop into a half-filled table (lines 20-24), then symmetrised in the `Prefix` (line 25). Reported (no symmetric matrix read).
- The leaf case (lines 37-46) is formatted into ten lines of nested calls, which hides "switch cost plus the flow to every disagreeing ancestor's other half".
- `let unseen: ?Int = None` (line 48) exists only to type `[unseen] * (...)`.

## Owed rewrites

- Name the leaf cost (`fn charge(i, b, flags)`), and type the table as `[0..=2 * half]?Int`.

## Language gaps

- Symmetric matrix read, a heap-node domain and a flag set over depths: in the friction.

## Dream notes

The typed signatures say what the port's comments say; the fragment stops short of the leaf charge and the combine.
