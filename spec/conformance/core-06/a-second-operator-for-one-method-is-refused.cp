// expect: E0202
// engines: next
// rule: 06.4.12
// R06.4.12: a second declaration of one `@operator` with the same method is E0202: core's `Eq`
// already marks `==`.
@operator("==") // <- here
trait Same:
    fn eq(self, other: Self) -> Bool
print(1)
