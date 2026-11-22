// error: `grow` can't be called here: it changes `xs`, and the `for` loop on line 6 is going over `xs`
var xs: List<Int> = List<Int>()
xs.push(1)
fn grow():
    xs.push(7)
for x in xs:
    grow()
print(xs.len)
