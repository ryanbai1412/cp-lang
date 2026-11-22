use std.derive.{Hash, hash_fields}

// `impl Hash for Int` and friends live in `std.derive`; a struct's `Hash` is `hash_fields` over them (decision 28.8).
@allow(unread_field)
struct Name(first: Str, last: Str)
impl Hash for Name:
    fn hash(self) = hash_fields(self)

@allow(unread_field)
struct Person(name: Name, age: Int, tag: Char, ok: Bool)
impl Hash for Person:
    fn hash(self) = hash_fields(self)

let p = Person(Name("ada", "l"), 36, 'x', true)
print(p.hash() == Person(Name("ada", "l"), 36, 'x', true).hash())
print(
    p.hash() == Person(Name("ada", "m"), 36, 'x', true).hash(),
    p.hash() == Person(Name("ad", "al"), 36, 'x', true).hash(),
)
print(hash_fields((1, "a")) == hash_fields((1, "a")), hash_fields((1, 2)) == hash_fields((2, 1)))
print(7.hash() % 1000, 'c'.hash() % 1000, "ab".hash() % 1000, p.hash() % 1000)
