// expect: ok, output "3"
// rule: 06.4.15
// `@word` names the word a type's constructor is; a run ignores it.
@word(boxed)
struct Box(v: Int)
print(Box(3).v)
