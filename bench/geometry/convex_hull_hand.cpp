#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

struct Pt {
    int64_t x, y;
    bool operator==(const Pt& o) const { return x == o.x && y == o.y; }
};

static inline int turn(const Pt& a, const Pt& b, const Pt& c) {
    __int128 k = (__int128)(b.x - a.x) * (c.y - a.y) - (__int128)(b.y - a.y) * (c.x - a.x);
    return (k > 0) - (k < 0);
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<Pt> pts(n);
    for (int i = 0; i < n; ++i) cin >> pts[i].x >> pts[i].y;

    sort(pts.begin(), pts.end(), [](const Pt& a, const Pt& b) {
        return a.x < b.x || (a.x == b.x && a.y < b.y);
    });
    pts.erase(unique(pts.begin(), pts.end()), pts.end());

    vector<Pt> h;
    if (pts.size() <= 2) {
        h = pts;
    } else {
        vector<Pt> lo, hi;
        for (const auto& p : pts) {
            while (lo.size() >= 2 && turn(lo[lo.size() - 2], lo.back(), p) <= 0) lo.pop_back();
            lo.push_back(p);
        }
        for (const auto& p : pts) {
            while (hi.size() >= 2 && turn(hi[hi.size() - 2], hi.back(), p) >= 0) hi.pop_back();
            hi.push_back(p);
        }
        h = lo;
        for (int i = (int)hi.size() - 2; i >= 1; --i) h.push_back(hi[i]);
    }
    int64_t checksum = 0;
    for (const auto& p : h) checksum += p.x + p.y;
    cout << h.size() << ' ' << checksum << '\n';
    return 0;
}
