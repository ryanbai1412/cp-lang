// Canada 2021 P6 3D Rage: a block's safeness is the least max(X - x, Y - y, Z - z) over the black
// holes it dominates, or -1. A k-d tree over the holes answers each block by branch and bound: a
// subtree is skipped when its box has no point the block dominates, or when the box's best
// possible value is no better than the answer so far. Exact; fast on the generated data.
#include <algorithm>
#include <array>
#include <cstdio>
#include <vector>

using P = std::array<int, 3>;
static std::vector<P> pts;
static std::vector<P> lo, hi;
static int X[3];
static int best;

static void build(int l, int r, int d) {
    if (l >= r) return;
    int m = (l + r) / 2;
    std::nth_element(pts.begin() + l, pts.begin() + m, pts.begin() + r,
                     [d](const P& a, const P& b) { return a[d] < b[d]; });
    build(l, m, (d + 1) % 3);
    build(m + 1, r, (d + 1) % 3);
    lo[m] = hi[m] = pts[m];
    for (int c : {(l + m) / 2, (m + 1 + r) / 2}) {
        if ((c < m && l < m) || (c > m && m + 1 < r))
            for (int k = 0; k < 3; k++) {
                lo[m][k] = std::min(lo[m][k], lo[c][k]);
                hi[m][k] = std::max(hi[m][k], hi[c][k]);
            }
    }
}

static int bound(int m) {
    int b = 0;
    for (int k = 0; k < 3; k++) {
        if (lo[m][k] > X[k]) return 1 << 30;
        b = std::max(b, X[k] - std::min(hi[m][k], X[k]));
    }
    return b;
}

static void query(int l, int r) {
    if (l >= r) return;
    int m = (l + r) / 2;
    if (bound(m) >= best) return;
    const P& p = pts[m];
    if (p[0] <= X[0] && p[1] <= X[1] && p[2] <= X[2])
        best = std::min(best, std::max({X[0] - p[0], X[1] - p[1], X[2] - p[2]}));
    int a = (l + m) / 2, b = (m + 1 + r) / 2;
    bool hasA = l < m, hasB = m + 1 < r;
    int ba = hasA ? bound(a) : 1 << 30, bb = hasB ? bound(b) : 1 << 30;
    if (ba <= bb) {
        if (hasA) query(l, m);
        if (hasB) query(m + 1, r);
    } else {
        if (hasB) query(m + 1, r);
        if (hasA) query(l, m);
    }
}

int main() {
    int n, q;
    if (std::scanf("%d %d", &n, &q) != 2) return 0;
    pts.resize(n);
    lo.resize(n);
    hi.resize(n);
    for (auto& p : pts) std::scanf("%d %d %d", &p[0], &p[1], &p[2]);
    build(0, n, 0);
    std::vector<char> out;
    out.reserve(12 * (size_t)q);
    char buf[16];
    while (q--) {
        std::scanf("%d %d %d", &X[0], &X[1], &X[2]);
        best = 1 << 30;
        query(0, n);
        int len = std::snprintf(buf, sizeof buf, "%d\n", best == 1 << 30 ? -1 : best);
        out.insert(out.end(), buf, buf + len);
    }
    std::fwrite(out.data(), 1, out.size(), stdout);
}
