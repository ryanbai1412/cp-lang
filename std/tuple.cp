//! Tuples `(a, b)`, as `Tuple1` to `Tuple12`: compared field by field, cloned deeply, and a
//! product domain when every part is a domain. Core re-exports them, so no module imports them.
//!
//! start: `Tuple2`, `Tuple3`, `Tuple1`
//! example: `(1, 2) < (1, 3) and (2, "a").clone() == (2, "a")`

/// A tuple of one value, field `_0`.
@role(tuple)
pub struct Tuple1<A>:
    pub _0: A

/// A tuple of two values, `(a, b)`, fields `_0` and `_1`.
@role(tuple)
pub struct Tuple2<A, B>:
    pub _0: A
    pub _1: B

/// A tuple of three values, `(a, b, c)`, fields `_0` to `_2`.
@role(tuple)
pub struct Tuple3<A, B, C>:
    pub _0: A
    pub _1: B
    pub _2: C

/// A tuple of 4 values, fields `_0` to `_3`.
@role(tuple)
pub struct Tuple4<A, B, C, E>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E

/// A tuple of 5 values, fields `_0` to `_4`.
@role(tuple)
pub struct Tuple5<A, B, C, E, F>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F

/// A tuple of 6 values, fields `_0` to `_5`.
@role(tuple)
pub struct Tuple6<A, B, C, E, F, G>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F
    pub _5: G

/// A tuple of 7 values, fields `_0` to `_6`.
@role(tuple)
pub struct Tuple7<A, B, C, E, F, G, H>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F
    pub _5: G
    pub _6: H

/// A tuple of 8 values, fields `_0` to `_7`.
@role(tuple)
pub struct Tuple8<A, B, C, E, F, G, H, J>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F
    pub _5: G
    pub _6: H
    pub _7: J

/// A tuple of 9 values, fields `_0` to `_8`.
@role(tuple)
pub struct Tuple9<A, B, C, E, F, G, H, J, K>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F
    pub _5: G
    pub _6: H
    pub _7: J
    pub _8: K

/// A tuple of 10 values, fields `_0` to `_9`.
@role(tuple)
pub struct Tuple10<A, B, C, E, F, G, H, J, K, L>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F
    pub _5: G
    pub _6: H
    pub _7: J
    pub _8: K
    pub _9: L

/// A tuple of 11 values, fields `_0` to `_10`.
@role(tuple)
pub struct Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F
    pub _5: G
    pub _6: H
    pub _7: J
    pub _8: K
    pub _9: L
    pub _10: M

/// A tuple of 12 values, fields `_0` to `_11`.
@role(tuple)
pub struct Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    pub _0: A
    pub _1: B
    pub _2: C
    pub _3: E
    pub _4: F
    pub _5: G
    pub _6: H
    pub _7: J
    pub _8: K
    pub _9: L
    pub _10: M
    pub _11: N

/// Tuples compare field by field, lexicographically for `Ord` (R08.1.6).
// A product domain walks row-major, the last factor fastest (R04.2.9); `P((x, y))` converts each part,
// and `Int(p)` is `p`'s ordinal (R04.2.15).
impl<A: Domain, B: Domain> Domain for Tuple2<A, B>:
    @get fn count() -> Int = A.count * B.count
    fn nth(k: Int) -> Tuple2<A, B> = (A.nth(k / B.count), B.nth(k % B.count))
    @get fn ord(self) -> Int = self._0.ord * B.count + self._1.ord
impl<A: Domain, B: Domain> Into<Tuple2<A, B>> for Tuple2<Int, Int>:
    fn into(self) -> Tuple2<A, B> = (A(self._0), B(self._1))
impl<A: Domain, B: Domain> Into<Int> for Tuple2<A, B>:
    fn into(self) -> Int = self.ord
impl<A: Ordered, B: Ordered> Ordered for Tuple2<A, B>

impl<A: Domain, B: Domain, C: Domain> Domain for Tuple3<A, B, C>:
    @get fn count() -> Int = A.count * B.count * C.count
    fn nth(k: Int) -> Tuple3<A, B, C> =
        (A.nth(k / (B.count * C.count)), B.nth(k / C.count % B.count), C.nth(k % C.count))
    @get fn ord(self) -> Int = (self._0.ord * B.count + self._1.ord) * C.count + self._2.ord
impl<A: Domain, B: Domain, C: Domain> Into<Tuple3<A, B, C>> for Tuple3<Int, Int, Int>:
    fn into(self) -> Tuple3<A, B, C> = (A(self._0), B(self._1), C(self._2))
impl<A: Domain, B: Domain, C: Domain> Into<Int> for Tuple3<A, B, C>:
    fn into(self) -> Int = self.ord
impl<A: Ordered, B: Ordered, C: Ordered> Ordered for Tuple3<A, B, C>
impl<A: Domain, B: Domain, C: Domain, E: Domain> Domain for Tuple4<A, B, C, E>:
    @get fn count() -> Int = A.count * B.count * C.count * E.count
    fn nth(k: Int) -> Tuple4<A, B, C, E>:
        let (c, e) = (C.count, E.count)
        (A.nth(k / (B.count * c * e)), B.nth(k / (c * e) % B.count), C.nth(k / e % c), E.nth(k % e))
    @get fn ord(self) -> Int =
        ((self._0.ord * B.count + self._1.ord) * C.count + self._2.ord) * E.count + self._3.ord
impl<A: Domain, B: Domain, C: Domain, E: Domain> Into<Tuple4<A, B, C, E>> for Tuple4<Int, Int, Int, Int>:
    fn into(self) -> Tuple4<A, B, C, E> = (A(self._0), B(self._1), C(self._2), E(self._3))
impl<A: Domain, B: Domain, C: Domain, E: Domain> Into<Int> for Tuple4<A, B, C, E>:
    fn into(self) -> Int = self.ord
impl<A: Ordered, B: Ordered, C: Ordered, E: Ordered> Ordered for Tuple4<A, B, C, E>

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

impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq> Eq for Tuple5<A, B, C, E, F>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord> Ord for Tuple5<A, B, C, E, F>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4))))
impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq, G: Eq> Eq for Tuple6<A, B, C, E, F, G>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4 and a._5 == b._5
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord, G: Ord> Ord for Tuple6<A, B, C, E, F, G>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4 or a._4 == b._4 and (a._5 < b._5)))))
impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq, G: Eq, H: Eq> Eq for Tuple7<A, B, C, E, F, G, H>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4 and a._5 == b._5 and a._6 == b._6
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord, G: Ord, H: Ord> Ord for Tuple7<A, B, C, E, F, G, H>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4 or a._4 == b._4 and (a._5 < b._5 or a._5 == b._5 and (a._6 < b._6))))))
impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq, G: Eq, H: Eq, J: Eq> Eq for Tuple8<A, B, C, E, F, G, H, J>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4 and a._5 == b._5 and a._6 == b._6 and a._7 == b._7
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord, G: Ord, H: Ord, J: Ord> Ord for Tuple8<A, B, C, E, F, G, H, J>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4 or a._4 == b._4 and (a._5 < b._5 or a._5 == b._5 and (a._6 < b._6 or a._6 == b._6 and (a._7 < b._7)))))))
impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq, G: Eq, H: Eq, J: Eq, K: Eq> Eq for Tuple9<A, B, C, E, F, G, H, J, K>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4 and a._5 == b._5 and a._6 == b._6 and a._7 == b._7 and a._8 == b._8
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord, G: Ord, H: Ord, J: Ord, K: Ord> Ord for Tuple9<A, B, C, E, F, G, H, J, K>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4 or a._4 == b._4 and (a._5 < b._5 or a._5 == b._5 and (a._6 < b._6 or a._6 == b._6 and (a._7 < b._7 or a._7 == b._7 and (a._8 < b._8))))))))
impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq, G: Eq, H: Eq, J: Eq, K: Eq, L: Eq> Eq for Tuple10<A, B, C, E, F, G, H, J, K, L>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4 and a._5 == b._5 and a._6 == b._6 and a._7 == b._7 and a._8 == b._8 and a._9 == b._9
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord, G: Ord, H: Ord, J: Ord, K: Ord, L: Ord> Ord for Tuple10<A, B, C, E, F, G, H, J, K, L>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4 or a._4 == b._4 and (a._5 < b._5 or a._5 == b._5 and (a._6 < b._6 or a._6 == b._6 and (a._7 < b._7 or a._7 == b._7 and (a._8 < b._8 or a._8 == b._8 and (a._9 < b._9)))))))))
impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq, G: Eq, H: Eq, J: Eq, K: Eq, L: Eq, M: Eq> Eq for Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4 and a._5 == b._5 and a._6 == b._6 and a._7 == b._7 and a._8 == b._8 and a._9 == b._9 and a._10 == b._10
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord, G: Ord, H: Ord, J: Ord, K: Ord, L: Ord, M: Ord> Ord for Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4 or a._4 == b._4 and (a._5 < b._5 or a._5 == b._5 and (a._6 < b._6 or a._6 == b._6 and (a._7 < b._7 or a._7 == b._7 and (a._8 < b._8 or a._8 == b._8 and (a._9 < b._9 or a._9 == b._9 and (a._10 < b._10))))))))))
impl<A: Eq, B: Eq, C: Eq, E: Eq, F: Eq, G: Eq, H: Eq, J: Eq, K: Eq, L: Eq, M: Eq, N: Eq> Eq for Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3 and a._4 == b._4 and a._5 == b._5 and a._6 == b._6 and a._7 == b._7 and a._8 == b._8 and a._9 == b._9 and a._10 == b._10 and a._11 == b._11
impl<A: Ord, B: Ord, C: Ord, E: Ord, F: Ord, G: Ord, H: Ord, J: Ord, K: Ord, L: Ord, M: Ord, N: Ord> Ord for Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and (a._3 < b._3 or a._3 == b._3 and (a._4 < b._4 or a._4 == b._4 and (a._5 < b._5 or a._5 == b._5 and (a._6 < b._6 or a._6 == b._6 and (a._7 < b._7 or a._7 == b._7 and (a._8 < b._8 or a._8 == b._8 and (a._9 < b._9 or a._9 == b._9 and (a._10 < b._10 or a._10 == b._10 and (a._11 < b._11)))))))))))

impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone> Clone for Tuple5<A, B, C, E, F>:
    fn clone(a) = Tuple5(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone, G: Clone> Clone for Tuple6<A, B, C, E, F, G>:
    fn clone(a) = Tuple6(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone(), a._5.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone, G: Clone, H: Clone> Clone for Tuple7<A, B, C, E, F, G, H>:
    fn clone(a) = Tuple7(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone(), a._5.clone(), a._6.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone, G: Clone, H: Clone, J: Clone> Clone for Tuple8<A, B, C, E, F, G, H, J>:
    fn clone(a) = Tuple8(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone(), a._5.clone(), a._6.clone(), a._7.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone, G: Clone, H: Clone, J: Clone, K: Clone> Clone for Tuple9<A, B, C, E, F, G, H, J, K>:
    fn clone(a) = Tuple9(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone(), a._5.clone(), a._6.clone(), a._7.clone(), a._8.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone, G: Clone, H: Clone, J: Clone, K: Clone, L: Clone> Clone for Tuple10<A, B, C, E, F, G, H, J, K, L>:
    fn clone(a) = Tuple10(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone(), a._5.clone(), a._6.clone(), a._7.clone(), a._8.clone(), a._9.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone, G: Clone, H: Clone, J: Clone, K: Clone, L: Clone, M: Clone> Clone for Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    fn clone(a) = Tuple11(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone(), a._5.clone(), a._6.clone(), a._7.clone(), a._8.clone(), a._9.clone(), a._10.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone, F: Clone, G: Clone, H: Clone, J: Clone, K: Clone, L: Clone, M: Clone, N: Clone> Clone for Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    fn clone(a) = Tuple12(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone(), a._4.clone(), a._5.clone(), a._6.clone(), a._7.clone(), a._8.clone(), a._9.clone(), a._10.clone(), a._11.clone())
