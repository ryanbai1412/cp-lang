# ioi09p5

Score: 4/5
Dream: 4/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 4/5, from the friction's `## Dream`. A `MinHeap` of free spots and a `Deque` of waiting cars, parked after each event (lines 17-31): the header's simulation.
- The signed event read and the `event > 0` test (lines 23-25) are in the friction (no sign-tagged enum read).
- `while waiting.len > 0 and free.len > 0` then two unwraps (lines 27-28) is in the friction.
