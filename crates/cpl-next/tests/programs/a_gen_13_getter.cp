// expect: ok, output "3"
// rule: R02.6.6
// A struct with a getter and a method of the same field-like use; getter read without parens.
struct Box:
    var xs: List<Int>
    @get fn size(self) -> Int:
        xs.len
var b = Box([1, 2])
b.xs.push(3)
print(b.size)
