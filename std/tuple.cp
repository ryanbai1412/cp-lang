//! Tuples `(a, b)`, as `Tuple1` to `Tuple4`: compared field by field, cloned deeply, and a
//! product domain when every part is a domain. Core re-exports them, so no module imports them.
//!
//! example: `(1, 2) < (1, 3) and (2, "a").clone() == (2, "a")`

@tuple
pub struct Tuple1<A>:
    _0: A

@tuple
pub struct Tuple2<A, B>:
    _0: A
    _1: B

@tuple
pub struct Tuple3<A, B, C>:
    _0: A
    _1: B
    _2: C

@tuple
pub struct Tuple4<A, B, C, E>:
    _0: A
    _1: B
    _2: C
    _3: E

/// Tuples compare field by field, lexicographically for `Ord` (R08.1.6).
// A product domain walks row-major, the last factor fastest (R04.2.9); `P((x, y))` converts each part.
impl<A: Domain, B: Domain> Domain for Tuple2<A, B>:
    @get fn count() -> Int = A.count * B.count
    fn nth(k: Int) -> Tuple2<A, B> = (A.nth(k / B.count), B.nth(k % B.count))
    @get fn ord(self) -> Int = self._0.ord * B.count + self._1.ord
impl<A: Domain, B: Domain> Into<Tuple2<A, B>> for Tuple2<Int, Int>:
    fn into(self) -> Tuple2<A, B> = (A(self._0), B(self._1))

impl<A: Domain, B: Domain, C: Domain> Domain for Tuple3<A, B, C>:
    @get fn count() -> Int = A.count * B.count * C.count
    fn nth(k: Int) -> Tuple3<A, B, C> =
        (A.nth(k / (B.count * C.count)), B.nth(k / C.count % B.count), C.nth(k % C.count))
    @get fn ord(self) -> Int = (self._0.ord * B.count + self._1.ord) * C.count + self._2.ord
impl<A: Domain, B: Domain, C: Domain> Into<Tuple3<A, B, C>> for Tuple3<Int, Int, Int>:
    fn into(self) -> Tuple3<A, B, C> = (A(self._0), B(self._1), C(self._2))
impl<A: Domain, B: Domain, C: Domain, E: Domain> Domain for Tuple4<A, B, C, E>:
    @get fn count() -> Int = A.count * B.count * C.count * E.count
    fn nth(k: Int) -> Tuple4<A, B, C, E>:
        let (c, e) = (C.count, E.count)
        (A.nth(k / (B.count * c * e)), B.nth(k / (c * e) % B.count), C.nth(k / e % c), E.nth(k % e))
    @get fn ord(self) -> Int =
        ((self._0.ord * B.count + self._1.ord) * C.count + self._2.ord) * E.count + self._3.ord
impl<A: Domain, B: Domain, C: Domain, E: Domain> Into<Tuple4<A, B, C, E>> for Tuple4<Int, Int, Int, Int>:
    fn into(self) -> Tuple4<A, B, C, E> = (A(self._0), B(self._1), C(self._2), E(self._3))

impl<A: Eq, B: Eq> Eq for Tuple2<A, B>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1
impl<A: Ord, B: Ord> Ord for Tuple2<A, B>:
    fn lt(a, b) = a._0 < b._0 or a._0 == b._0 and a._1 < b._1
impl<A: Eq, B: Eq, C: Eq> Eq for Tuple3<A, B, C>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2
impl<A: Ord, B: Ord, C: Ord> Ord for Tuple3<A, B, C>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and a._2 < b._2)
impl<A: Eq, B: Eq, C: Eq, E: Eq> Eq for Tuple4<A, B, C, E>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3
impl<A: Ord, B: Ord, C: Ord, E: Ord> Ord for Tuple4<A, B, C, E>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and a._3 < b._3))

/// `x.clone()` on a tuple clones each field (R05.2.2).
impl<A: Clone, B: Clone> Clone for Tuple2<A, B>:
    fn clone(a) = Tuple2(a._0.clone(), a._1.clone())
impl<A: Clone, B: Clone, C: Clone> Clone for Tuple3<A, B, C>:
    fn clone(a) = Tuple3(a._0.clone(), a._1.clone(), a._2.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone> Clone for Tuple4<A, B, C, E>:
    fn clone(a) = Tuple4(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone())

/// `max` and `min` past their first two values (R11.3.3).
impl<T: Ord> Alike<T> for Tuple1<T>:
    fn max_with(self, m: T) -> T = max(m, self._0)
    fn min_with(self, m: T) -> T = min(m, self._0)
impl<T: Ord> Alike<T> for Tuple2<T, T>:
    fn max_with(self, m: T) -> T = max(m, self._0, self._1)
    fn min_with(self, m: T) -> T = min(m, self._0, self._1)
impl<T: Ord> Alike<T> for Tuple3<T, T, T>:
    fn max_with(self, m: T) -> T = max(m, self._0, self._1, self._2)
    fn min_with(self, m: T) -> T = min(m, self._0, self._1, self._2)
impl<T: Ord> Alike<T> for Tuple4<T, T, T, T>:
    fn max_with(self, m: T) -> T = max(m, self._0, self._1, self._2, self._3)
    fn min_with(self, m: T) -> T = min(m, self._0, self._1, self._2, self._3)
