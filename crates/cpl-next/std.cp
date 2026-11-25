fn raw_empty<T>() -> Raw<T> = @intrinsic("raw_empty")
fn raw_new<T>(n: Int, fill: T) -> Raw<T> = @intrinsic("raw_new")
fn raw_get<T>(r: Raw<T>, i: Int) -> T = @intrinsic("raw_get")
fn raw_set<T>(mut r: Raw<T>, i: Int, x: T) = @intrinsic("raw_set")

struct List<T>:
    var buf: Raw<T>
    var n: Int
    var cap: Int

    fn new() -> List<T>:
        List(raw_empty(), 0, 0)

    fn filled(n: Int, x: T) -> List<T>:
        List(raw_new(n, x), n, n)

    @get fn len(self) -> Int:
        n

    fn get(self, i: Int) -> T:
        assert 0 <= i and i < n, "index out of range"
        raw_get(buf, i)

    fn set(mut self, i: Int, x: T):
        assert 0 <= i and i < n, "index out of range"
        raw_set(mut buf, i, x)

    fn each(self) -> yields T:
        for i in 0..n: yield raw_get(buf, i)

    fn push(mut self, x: T):
        if n == cap:
            let ncap = if cap == 0: 4 else: cap * 2
            var nb = raw_new(ncap, x)
            for i in 0..n: raw_set(mut nb, i, raw_get(buf, i))
            buf = nb
            cap = ncap
        raw_set(mut buf, n, x)
        n += 1


struct Array<D, T>:
    buf: Raw<T>

    fn filled(x: T) -> Array<D, T>:
        Array(raw_new(D.count, x))

    fn get(self, i: D) -> T:
        raw_get(buf, i.ord)

    fn set(mut self, i: D, x: T):
        raw_set(mut buf, i.ord, x)

    fn each(self) -> yields T:
        for i in 0..D.count: yield raw_get(buf, i)

fn read_array<D>() -> [D]Int:
    var a: [D]Int = 0
    for v in D: a[v] = read()
    a

fn write_list(xs: List<Int>):
    var first = true
    for x in xs:
        if not first: write(" ")
        write(x)
        first = false

fn write_array<D>(xs: [D]Int):
    var first = true
    for x in xs:
        if not first: write(" ")
        write(x)
        first = false

// A flat sequence of `Char` (R02.3).
struct Str:
    var chars: List<Char>

    @get fn len(self) -> Int:
        chars.len

    fn get(self, i: Int) -> Char:
        chars.get(i)

    fn set(mut self, i: Int, c: Char):
        chars.set(i, c)

    fn each(self) -> yields Char:
        for c in chars: yield c

    fn push(mut self, c: Char):
        chars.push(c)

// A string literal: its characters and how many there are. The literal's storage is lent,
// so the string gets its own.
fn str_of(text: Raw<Char>, n: Int) -> Str:
    var chars = List<Char>.new()
    for i in 0..n: chars.push(raw_get(text, i))
    Str(chars)

fn str_concat(a: Str, b: Str) -> Str:
    var out = Str(List.new())
    for c in a: out.push(c)
    for c in b: out.push(c)
    out

fn str_eq(a: Str, b: Str) -> Bool:
    if a.len != b.len: return false
    for i in 0..a.len:
        if a.get(i) != b.get(i): return false
    true

fn write_str(s: Str):
    for c in s: write(c)

fn yes(b: Bool) -> Str:
    if b: "YES" else: "NO"

// `?T`: `T` or `None`. `val` is read only when `some`.
struct Optional<T>:
    some: Bool
    val: T

struct Tuple2<A, B>:
    _0: A
    _1: B

struct Tuple3<A, B, C>:
    _0: A
    _1: B
    _2: C

struct Tuple4<A, B, C, E>:
    _0: A
    _1: B
    _2: C
    _3: E
