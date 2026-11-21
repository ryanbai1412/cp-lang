#include <bits/stdc++.h>
using namespace std;

struct Pt {
    long long x, y;
    Pt operator-(Pt o) const { return {x - o.x, y - o.y}; }
};

static inline __int128 dot128(Pt a, Pt b) { return (__int128)a.x * b.x + (__int128)a.y * b.y; }
static inline __int128 cross128(Pt a, Pt b) { return (__int128)a.x * b.y - (__int128)a.y * b.x; }

static pair<double, double> min_bounding_box(const vector<Pt> &hull) {
    int n = (int)hull.size();
    if (n <= 1) return {0.0, 0.0};
    double best_area = 1e300, best_perim = 1e300;
    int right = 1, top = 1, left = 1;
    for (int i = 0; i < n; i++) {
        int ni = (i + 1 == n) ? 0 : i + 1;
        Pt e = hull[ni] - hull[i];
        if (e.x == 0 && e.y == 0) continue;
        while (true) {
            int nr = (right + 1 == n) ? 0 : right + 1;
            if (dot128(e, hull[nr] - hull[right]) > 0) right = nr;
            else break;
        }
        while (true) {
            int nt = (top + 1 == n) ? 0 : top + 1;
            if (cross128(e, hull[nt] - hull[top]) > 0) top = nt;
            else break;
        }
        if (i == 0) left = top;
        while (true) {
            int nl = (left + 1 == n) ? 0 : left + 1;
            if (dot128(e, hull[nl] - hull[left]) < 0) left = nl;
            else break;
        }
        double elen = sqrt((double)dot128(e, e));
        double w = (double)dot128(e, hull[right] - hull[left]) / elen;
        double h = (double)cross128(e, hull[top] - hull[i]) / elen;
        best_area = min(best_area, w * h);
        best_perim = min(best_perim, 2.0 * (w + h));
    }
    return {best_area, best_perim};
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    cin >> n;
    vector<Pt> hull(n);
    for (int i = 0; i < n; i++) cin >> hull[i].x >> hull[i].y;
    double sum_a = 0.0, sum_p = 0.0;
    for (int r = 0; r < 50; r++) {
        auto [a, p] = min_bounding_box(hull);
        sum_a += a;
        sum_p += p;
    }
    cout << llround(sum_a / 1e12) << ' ' << llround(sum_p / 1e4) << '\n';
}
