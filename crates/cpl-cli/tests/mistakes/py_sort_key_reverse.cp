// here: 4
// Python's `reverse=True` with a key (decision 290 dropped `desc:`): the fix wrote `desc: true`.
var ws = ["bb", "a", "ccc"]
ws.sort(by: w => w.len, reverse: true)
print(ws)
