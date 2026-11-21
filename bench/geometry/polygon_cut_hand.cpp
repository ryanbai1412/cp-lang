#include <bits/stdc++.h>
using namespace std;

struct Vec {
    double x, y;
    Vec operator+(Vec o) const { return {x + o.x, y + o.y}; }
    Vec operator-(Vec o) const { return {x - o.x, y - o.y}; }
    Vec operator*(double k) const { return {x * k, y * k}; }
};

static inline double vcross(Vec a, Vec b) { return a.x * b.y - a.y * b.x; }

static vector<Vec> polygon_cut(const vector<Vec>& poly, Vec a, Vec b, double eps = 1e-9) {
    int n = (int)poly.size();
    if (n == 0) return {};
    Vec d = b - a;
    vector<Vec> out;
    out.reserve(n + 2);
    for (int i = 0; i < n; i++) {
        Vec cur = poly[i], nxt = poly[(i + 1) % n];
        double sc = vcross(d, cur - a);
        double sn = vcross(d, nxt - a);
        if (sc >= -eps) out.push_back(cur);
        if ((sc > eps && sn < -eps) || (sc < -eps && sn > eps)) {
            double t = sc / (sc - sn);
            out.push_back(cur + (nxt - cur) * t);
        }
    }
    return out;
}

static double polygon_area(const vector<Vec>& poly) {
    int n = (int)poly.size();
    if (n < 3) return 0.0;
    double s = 0.0;
    for (int i = 0; i < n; i++) s += vcross(poly[i], poly[(i + 1) % n]);
    return 0.5 * s;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<Vec> poly = {{-10000.0, -10000.0}, {10000.0, -10000.0}, {10000.0, 10000.0}, {-10000.0, 10000.0}};
    for (int i = 0; i < n; i++) {
        double ax, ay, bx, by;
        cin >> ax >> ay >> bx >> by;
        poly = polygon_cut(poly, {ax, ay}, {bx, by});
    }
    cout << poly.size() << ' ' << llround(polygon_area(poly) * 1000.0) << '\n';
}
