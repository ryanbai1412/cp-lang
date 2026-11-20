// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: ioi13p6 (Game — 2D Dynamic Segment Tree with GCD)
// Original: corpus/submissions/4688844.cpp
// Exercises: arena pattern (type Id + List<Node>.push_id), recursive `a[i].f = rhs`, ?Id instead of 0 sentinel, gcd, @grader struct

const MAX_NODES = 12_000_000
type InnerId = 0..=MAX_NODES
type OuterId = 0..=MAX_NODES

struct InnerNode:
    lc: ?InnerId = none
    rc: ?InnerId = none
    g: Int = 0

struct OuterNode:
    lc: ?OuterId = none
    rc: ?OuterId = none
    inner: ?InnerId = none

// BLIND-GUESS(H12): `@grader struct` stateful grader interface
@grader
struct Game:
    rows: Int = 0
    cols: Int = 0
    root: ?OuterId = none
    outer: List<OuterNode> = []
    inner: List<InnerNode> = []

    fn init(mut self, r: Int, c: Int):
        self.rows = r
        self.cols = c
        self.root = none
        self.outer.clear()
        self.inner.clear()

    fn inner_gcd(self, o: ?InnerId) -> Int:
        if let id = o: self.inner[id].g else: 0

    fn query_inner(self, o: ?InnerId, l: Int, r: Int, ql: Int, qr: Int) -> Int:
        let id = o? ?? return 0  // BLIND-GUESS: can't use `?` in fn returning Int! Must use `if let`!
        return 0

    fn query_y(self, o: ?InnerId, l: Int, r: Int, ql: Int, qr: Int) -> Int:
        if let id = o:
            if ql <= l and r <= qr:
                return self.inner[id].g
            let mid = (l + r) / 2
            var ans = 0
            if ql <= mid:
                ans = gcd(ans, self.query_y(self.inner[id].lc, l, mid, ql, qr))
            if mid < qr:
                ans = gcd(ans, self.query_y(self.inner[id].rc, mid + 1, r, ql, qr))
            return ans
        return 0

    fn query_x(self, o: ?OuterId, l: Int, r: Int, qx1: Int, qx2: Int, qy1: Int, qy2: Int) -> Int:
        if let id = o:
            if qx1 <= l and r <= qx2:
                return self.query_y(self.outer[id].inner, 0, self.cols - 1, qy1, qy2)
            let mid = (l + r) / 2
            var ans = 0
            if qx1 <= mid:
                ans = gcd(ans, self.query_x(self.outer[id].lc, l, mid, qx1, qx2, qy1, qy2))
            if mid < qx2:
                ans = gcd(ans, self.query_x(self.outer[id].rc, mid + 1, r, qx1, qx2, qy1, qy2))
            return ans
        return 0

    fn update_y(mut self, o: ?InnerId, l: Int, r: Int, y: Int, val: Int) -> InnerId:
        // BLIND-GUESS(H15): does push_id() -> Int implicitly narrow to InnerId (`0..=MAX_NODES`)?
        let id: InnerId = o ?? self.inner.push_id(InnerNode())
        if l == r:
            self.inner[id].g = val
            return id
        let mid = (l + r) / 2
        // Safe recursive arena update: v2.1 §6 evaluates RHS before borrowing self.inner[id]!
        if y <= mid:
            self.inner[id].lc = self.update_y(self.inner[id].lc, l, mid, y, val)
        else:
            self.inner[id].rc = self.update_y(self.inner[id].rc, mid + 1, r, y, val)
        self.inner[id].g = gcd(self.inner_gcd(self.inner[id].lc), self.inner_gcd(self.inner[id].rc))
        return id

    fn update_x(mut self, o: ?OuterId, l: Int, r: Int, x: Int, y: Int, val: Int) -> OuterId:
        let id: OuterId = o ?? self.outer.push_id(OuterNode())
        if l == r:
            self.outer[id].inner = self.update_y(self.outer[id].inner, 0, self.cols - 1, y, val)
            return id
        let mid = (l + r) / 2
        if x <= mid:
            self.outer[id].lc = self.update_x(self.outer[id].lc, l, mid, x, y, val)
        else:
            self.outer[id].rc = self.update_x(self.outer[id].rc, mid + 1, r, x, y, val)
        let lv = if let lc = self.outer[id].lc: self.query_y(self.outer[lc].inner, 0, self.cols - 1, y, y) else: 0
        let rv = if let rc = self.outer[id].rc: self.query_y(self.outer[rc].inner, 0, self.cols - 1, y, y) else: 0
        self.outer[id].inner = self.update_y(self.outer[id].inner, 0, self.cols - 1, y, gcd(lv, rv))
        return id

    fn update(mut self, p: Int, q: Int, k: Int):
        self.root = self.update_x(self.root, 0, self.rows - 1, p, q, k)

    fn calculate(self, p: Int, q: Int, u: Int, v: Int) -> Int:
        return self.query_x(self.root, 0, self.rows - 1, p, u, q, v)
