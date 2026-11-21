#include <bits/stdc++.h>
using namespace std;

struct Vec {
    double x, y;
    Vec operator+(Vec o) const { return {x + o.x, y + o.y}; }
    Vec operator-(Vec o) const { return {x - o.x, y - o.y}; }
    Vec operator*(double k) const { return {x * k, y * k}; }
    Vec operator/(double k) const { return {x / k, y / k}; }
    Vec perp() const { return {-y, x}; }
    double norm2() const { return x * x + y * y; }
    double len() const { return sqrt(norm2()); }
};

static vector<Vec> circle_tangents(Vec c, double r, Vec p, double eps = 1e-9) {
    Vec dv = p - c;
    double d = dv.len();
    if (d + eps < r || d <= eps) return {};
    if (d - r <= eps) return {c + dv * (r / d)};
    double d2 = dv.norm2(), r2 = r * r;
    double h = sqrt(max(0.0, d2 - r2));
    Vec base = c + dv * (r2 / d2);
    Vec off = dv.perp() * (r * h / d2);
    return {base - off, base + off};
}

static vector<pair<Vec, Vec>> circle_circle_tangents(Vec c1, double r1, Vec c2, double r2, bool inner, double eps = 1e-9) {
    Vec dv = c2 - c1;
    double d2 = dv.norm2();
    if (d2 <= eps * eps) return {};
    double r2s = inner ? -r2 : r2;
    double dr = r1 - r2s;
    double h2 = d2 - dr * dr;
    if (h2 < -eps * max(1.0, d2)) return {};
    double h = sqrt(max(0.0, h2));
    if (h <= eps) {
        Vec v = dv * (dr / d2);
        return {{c1 + v * r1, c2 + v * r2s}};
    }
    Vec v1 = (dv * dr - dv.perp() * h) / d2;
    Vec v2 = (dv * dr + dv.perp() * h) / d2;
    return {{c1 + v1 * r1, c2 + v1 * r2s}, {c1 + v2 * r1, c2 + v2 * r2s}};
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    cin >> n;
    long long cnt = 0;
    double acc = 0.0;
    for (int i = 0; i < n; i++) {
        double x1, y1, r1, x2, y2, r2;
        cin >> x1 >> y1 >> r1 >> x2 >> y2 >> r2;
        Vec c1{x1, y1}, c2{x2, y2};
        auto pt_ts = circle_tangents(c1, r1, c2);
        cnt += (long long)pt_ts.size();
        for (auto &p : pt_ts) acc += p.x + p.y;
        auto out_ts = circle_circle_tangents(c1, r1, c2, r2, false);
        cnt += (long long)out_ts.size();
        for (auto &[a, b] : out_ts) acc += a.x + b.y;
        auto in_ts = circle_circle_tangents(c1, r1, c2, r2, true);
        cnt += (long long)in_ts.size();
        for (auto &[a, b] : in_ts) acc += a.x + b.y;
    }
    cout << cnt << ' ' << llround(acc * 100.0) << '\n';
}
