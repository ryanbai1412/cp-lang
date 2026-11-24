# cheerio1s2 frictions

## Dream

```cp
var next = MinHeap((1..=n).map(i => a * i + b))
print(sum(next.pop_push(w => w + b) for _ in 0..n))
```

## Heap from a sequence

`MinHeap` takes no initial sequence, so the n floor costs are pushed in a loop, and a pop that
pushes its successor is spelled as two calls.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let n = 3
var next = MinHeap((1..=n).map(i => 2 * i + 1))
print(next.pop()!)
```

```text
error[E0302]: expected `?(Int) -> Int`, found `List<Int>`
  --> probe.cp:2:20
2 | var next = MinHeap((1..=n).map(i => 2 * i + 1))
  |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

**Status:** open: no `MinHeap` from a sequence.

```cp
var next = MinHeap<Int>()
next.push(3)
print(next.pop_push(w => w + 1))
```

```text
error[E0203]: Heap<T> has no field or method `pop_push`
  --> probe.cp:3:12
3 | print(next.pop_push(w => w + 1))
  |            ^^^^^^^^
  = help: its methods are `clear`, `drain`, `each`, `is_empty`, `len`, `peek`, `pop`, `push`, `push_all`, `push_pop` and 3 more
```

**Status:** open: std has `push_pop`, not pop-then-push.
