// error: `@!multitest` stands at the top level of the file
let n: 1..=9 = read()
if n > 1:
    @!multitest
    print(n)
