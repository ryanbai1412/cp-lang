#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

struct Item {
    int64_t x, y;
    int id;
};

static inline int64_t dist2(const Item& a, const Item& b) {
    int64_t dx = a.x - b.x, dy = a.y - b.y;
    return dx * dx + dy * dy;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<Item> a(n);
    for (int i = 0; i < n; ++i) {
        cin >> a[i].x >> a[i].y;
        a[i].id = i;
    }
    sort(a.begin(), a.end(), [](const Item& u, const Item& v) {
        return u.x < v.x || (u.x == v.x && u.y < v.y);
    });
    vector<Item> buf(n), strip;
    strip.reserve(n);
    int64_t best = dist2(a[0], a[1]);

    auto rec = [&](auto&& self, int l, int r) -> void {
        if (r - l <= 3) {
            for (int i = l; i < r; ++i)
                for (int j = i + 1; j < r; ++j)
                    best = min(best, dist2(a[i], a[j]));
            sort(a.begin() + l, a.begin() + r, [](const Item& u, const Item& v) { return u.y < v.y; });
            return;
        }
        int m = (l + r) / 2;
        int64_t mid_x = a[m].x;
        self(self, l, m);
        self(self, m, r);
        int i = l, j = m, k = l;
        while (i < m && j < r) buf[k++] = (a[i].y <= a[j].y) ? a[i++] : a[j++];
        while (i < m) buf[k++] = a[i++];
        while (j < r) buf[k++] = a[j++];
        for (int t = l; t < r; ++t) a[t] = buf[t];

        strip.clear();
        for (int t = l; t < r; ++t) {
            int64_t dx = a[t].x - mid_x;
            if ((__int128)dx * dx < best) strip.push_back(a[t]);
        }
        int sz = (int)strip.size();
        for (int u = 0; u < sz; ++u) {
            for (int v = u + 1; v < sz; ++v) {
                int64_t dy = strip[v].y - strip[u].y;
                if ((__int128)dy * dy >= best) break;
                best = min(best, dist2(strip[u], strip[v]));
            }
        }
    };
    rec(rec, 0, n);
    cout << best << '\n';
    return 0;
}
