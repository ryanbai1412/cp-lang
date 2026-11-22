// expect: ok, output "ok"
// rule: 05.3.1

struct Box:
    raw: Str
    fn parse_as<T>(self) -> ?T:
        raw.parse<T>()

let b = Box(raw: "ok")
let s: ?Str = b.parse_as()
print(s!)
