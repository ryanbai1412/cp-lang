// expect: ok
// rule: trace.7
// tool: run --trace --format json
// has: "\"b\":true"
// has: "\"o\":null"
// has: "\"o\":2"
// A `Bool` and a `?T` are not `Show`, and still show in a trace: `b` as `true`, `o` as its value or none.
var b = false
var o: ?Int = None
for i in 0..3:
    b = i == 2
    o = Some(i)
print(1)
