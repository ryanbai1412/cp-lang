#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

struct Pt { int64_t x, y; };

static inline int turn(const Pt& a, const Pt& b, const Pt& c) {
    __int128 k = (__int128)(b.x - a.x) * (c.y - a.y) - (__int128)(b.y - a.y) * (c.x - a.x);
    return (k > 0) - (k < 0);
}
static inline bool on_seg(const Pt& p, const Pt& a, const Pt& b) {
    if ((__int128)(a.x - p.x) * (b.y - p.y) != (__int128)(a.y - p.y) * (b.x - p.x)) return false;
    return (__int128)(a.x - p.x) * (b.x - p.x) + (__int128)(a.y - p.y) * (b.y - p.y) <= 0;
}
static inline bool seg_meet(const Pt& a, const Pt& b, const Pt& c, const Pt& d) {
    int t1 = turn(a, b, c), t2 = turn(a, b, d), t3 = turn(c, d, a), t4 = turn(c, d, b);
    if (t1 * t2 < 0 && t3 * t4 < 0) return true;
    return on_seg(c, a, b) || on_seg(d, a, b) || on_seg(a, c, d) || on_seg(b, c, d);
}

static inline int seg_cmp(const Pt& lo1, const Pt& hi1, int id1,
                          const Pt& lo2, const Pt& hi2, int id2,
                          int64_t cur_x, bool right_eps) {
    __int128 dx1 = hi1.x - lo1.x, dy1 = hi1.y - lo1.y;
    __int128 dx2 = hi2.x - lo2.x, dy2 = hi2.y - lo2.y;
    __int128 lhs = (__int128)lo1.y * dx1 * dx2 + dy1 * (cur_x - lo1.x) * dx2;
    __int128 rhs = (__int128)lo2.y * dx1 * dx2 + dy2 * (cur_x - lo2.x) * dx1;
    if (lhs != rhs) return lhs < rhs ? -1 : 1;
    __int128 slope = dy1 * dx2 - dy2 * dx1;
    if (slope != 0) {
        int s = slope < 0 ? -1 : 1;
        return right_eps ? s : -s;
    }
    return id1 < id2 ? -1 : (id1 > id2 ? 1 : 0);
}

struct Event { int64_t x, y; int kind, id; };

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<Pt> lo(n), hi(n);
    vector<Event> events;
    events.reserve(2 * n);
    for (int i = 0; i < n; ++i) {
        Pt a, b;
        cin >> a.x >> a.y >> b.x >> b.y;
        if (a.x > b.x || (a.x == b.x && a.y > b.y)) swap(a, b);
        lo[i] = a; hi[i] = b;
        if (a.x == b.x) {
            events.push_back({a.x, a.y, 1, i});
        } else {
            events.push_back({a.x, a.y, 0, i});
            events.push_back({b.x, b.y, 2, i});
        }
    }
    sort(events.begin(), events.end(), [](const Event& a, const Event& b) {
        if (a.x != b.x) return a.x < b.x;
        if (a.kind != b.kind) return a.kind < b.kind;
        if (a.y != b.y) return a.y < b.y;
        return a.id < b.id;
    });

    vector<int> lch(n, -1), rch(n, -1), prio(n, 0);
    int64_t rng = 123456789;
    int root = -1;

    auto merge = [&](auto&& self, int a, int b) -> int {
        if (a == -1) return b;
        if (b == -1) return a;
        if (prio[a] > prio[b]) {
            rch[a] = self(self, rch[a], b);
            return a;
        } else {
            lch[b] = self(self, a, lch[b]);
            return b;
        }
    };
    auto split_lt = [&](auto&& self, int u, int id, int64_t cx, bool reps) -> pair<int, int> {
        if (u == -1) return {-1, -1};
        if (seg_cmp(lo[u], hi[u], u, lo[id], hi[id], id, cx, reps) < 0) {
            auto [rl, rr] = self(self, rch[u], id, cx, reps);
            rch[u] = rl;
            return {u, rr};
        } else {
            auto [ll, lr] = self(self, lch[u], id, cx, reps);
            lch[u] = lr;
            return {ll, u};
        }
    };
    auto split_le = [&](auto&& self, int u, int id, int64_t cx, bool reps) -> pair<int, int> {
        if (u == -1) return {-1, -1};
        if (seg_cmp(lo[u], hi[u], u, lo[id], hi[id], id, cx, reps) <= 0) {
            auto [rl, rr] = self(self, rch[u], id, cx, reps);
            rch[u] = rl;
            return {u, rr};
        } else {
            auto [ll, lr] = self(self, lch[u], id, cx, reps);
            lch[u] = lr;
            return {ll, u};
        }
    };
    auto leftmost = [&](int u) {
        if (u == -1) return -1;
        while (lch[u] != -1) u = lch[u];
        return u;
    };
    auto rightmost = [&](int u) {
        if (u == -1) return -1;
        while (rch[u] != -1) u = rch[u];
        return u;
    };

    bool found = false;
    for (const auto& ev : events) {
        int id = ev.id;
        if (ev.kind == 0) {
            auto [l, r] = split_lt(split_lt, root, id, ev.x, true);
            int pred = rightmost(l), succ = leftmost(r);
            if (pred != -1 && seg_meet(lo[pred], hi[pred], lo[id], hi[id])) { found = true; break; }
            if (succ != -1 && seg_meet(lo[succ], hi[succ], lo[id], hi[id])) { found = true; break; }
            rng = (rng * 1103515245 + 12345) % 2147483647;
            prio[id] = (int)rng;
            lch[id] = rch[id] = -1;
            root = merge(merge, merge(merge, l, id), r);
        } else if (ev.kind == 2) {
            auto [l, midr] = split_lt(split_lt, root, id, ev.x, false);
            auto [mid, r] = split_le(split_le, midr, id, ev.x, false);
            int pred = rightmost(l), succ = leftmost(r);
            if (pred != -1 && succ != -1 && seg_meet(lo[pred], hi[pred], lo[succ], hi[succ])) { found = true; break; }
            root = merge(merge, l, r);
        }
    }
    cout << (found ? 1 : 0) << '\n';
    return 0;
}
