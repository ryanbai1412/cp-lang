// `std.derive` writes the derived `Eq`, `Ord` and `Show` on `fields`; an impl can call it (decision 28.7).
use std.derive.{eq_fields, lt_fields, show_fields}

struct Job(due: Int, name: Str)
impl Ord for Job:
    fn lt(a, b) = lt_fields(b, a)

struct Tag(id: Int, label: Str)
impl Show for Tag:
    fn show(a) = "#" + show_fields(a)

// Explicit comparisons, not `sort`: how often a sort calls `lt` differs between interpreter and C++.
print(
    Job(3, "a") < Job(1, "a"),
    Job(1, "a") < Job(3, "a"),
    Job(3, "c") < Job(3, "a"),
    Job(2, "b") < Job(2, "b"),
)
print(
    eq_fields(Job(1, "a"), Job(1, "a")),
    eq_fields(Job(1, "a"), Job(1, "b")),
    lt_fields(Job(1, "z"), Job(2, "a")),
)
print(Tag(7, "x"), show_fields((1, 'c', "s")))
