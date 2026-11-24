# cheerio1s2 frictions

## Dream

```cp
var next = MinHeap((1..=n).map(i => a * i + b))
print(sum(next.pop_push(w => w + b) for _ in 0..n))
```

## Heap from a sequence

`MinHeap` takes no initial sequence, so the n floor costs are pushed in a loop, and a pop that
pushes its successor is spelled as two calls.
