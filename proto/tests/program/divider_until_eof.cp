// `@!multitest(until_eof)` divides the file the same way; there is no count to read.
var seen = 0
@!multitest(until_eof)
let n: 0..=9 = read()
seen += 1
if n == 0:
    return
print(n, seen)
