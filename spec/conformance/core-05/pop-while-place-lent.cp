// expect: E0602 at 4
// rule: 05.8.4
// `pop` and `clear` change the length like `push`, even when their result is used as a value.
var rows = [[1, 2], [3]]
let r = mut rows[0]
let gone = rows.pop()
r.push(5)
print(rows, gone)
