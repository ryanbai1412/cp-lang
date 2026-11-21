#include <bits/stdc++.h>
using namespace std;

struct Vec {
    double x, y;
    Vec operator+(Vec o) const { return {x + o.x, y + o.y}; }
    Vec operator-(Vec o) const { return {x - o.x, y - o.y}; }
    Vec operator*(double k) const { return {x * k, y * k}; }
    double len() const { return sqrt(x * x + y * y); }
    Vec perp() const { return {-y, x}; }
};

static vector<Vec> circle_intersect(Vec c1, double r1, Vec c2, double r2, double eps = 1e-9) {
    Vec dv = c2 - c1;
    double d = dv.len();
    if (d <= eps) return {};
    if (d > r1 + r2 + eps || d + min(r1, r2) + eps < max(r1, r2)) return {};
    double x = (d * d + r1 * r1 - r2 * r2) / (2.0 * d);
    double h = sqrt(max(0.0, r1 * r1 - x * x));
    Vec base = c1 + dv * (x / d);
    if (h <= eps) return {base};
    Vec off = dv.perp() * (h / d);
    return {base - off, base + off};
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    long long cnt = 0;
    double acc = 0.0;
    for (int i = 0; i < n; i++) {
        double x1, y1, r1, x2, y2, r2;
        cin >> x1 >> y1 >> r1 >> x2 >> y2 >> r2;
        auto pts = circle_intersect({x1, y1}, r1, {x2, y2}, r2);
        cnt += (long long)pts.size();
        for (const auto& p : pts) acc += p.x + p.y;
    }
    cout << cnt << ' ' << llround(acc * 100.0) << '\n';
}
