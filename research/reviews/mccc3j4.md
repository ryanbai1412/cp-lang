# mccc3j4

Score: 4/5
Dream: 3/5

Cold review: a deque per player, each round's hits collected before any is appended (lines 18-25), as the header says.

## Checklist failures

- R8: `for _ in 0..t` (line 18) where `repeat t:` is the spelling; `repeat` exists today (`cpl run` of `repeat n: print(1)` prints three lines), so the friction's "There is no `repeat t:`" is wrong.

## Owed rewrites

- `repeat t:` for the round loop.

## Language gaps

- The filter-map comprehension over an optional is in research/frictions/mccc3j4.md; its `repeat` section is corrected.

## Dream notes

- `[(to: x, from: p) for p in Player if let x = queue[p].pop_front()]` pops inside a comprehension, a side effect hidden in an expression (R7), and `last[h.from] = h.to; queue[h.to].push(h.from)` joins two statements with `;`. The port's explicit loop is the better form; the dream should keep it.
