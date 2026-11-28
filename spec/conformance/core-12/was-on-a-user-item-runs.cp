// expect: ok, output "3"
// rule: 12.13.1
// `@was` names what an item replaced; any library may write it, and a run ignores it.
@was(OldBox)
struct Box(v: Int)
print(Box(3).v)
