// expect: ok, output "1\n2\n0\nyes\n3"
// A string literal pattern matches the strings its type's `Eq` calls equal (R01.7).
fn code(s: Str) -> Int =
    match s:
        "one" => 1
        "two" | "deux" => 2
        _ => 0
print(code("one"))
print(code("deux"))
print(code("three"))
let w = "go"
if let "go" = w: print("yes")
fn pair(p: (Str, Int)) -> Int =
    match p:
        ("a", n) => n
        (_, n) => n + 1
print(pair(("a", 1)) + pair(("b", 1)))
