#include <algorithm>
#include <cmath>
#include <cstdint>
#include <deque>
#include <iostream>
#include <vector>
using namespace std;

struct Vec {
    double x, y;
    Vec operator+(const Vec& o) const { return {x + o.x, y + o.y}; }
    Vec operator-(const Vec& o) const { return {x - o.x, y - o.y}; }
    Vec operator*(double k) const { return {x * k, y * k}; }
    Vec operator/(double k) const { return {x / k, y / k}; }
};

static inline double vdot(const Vec& a, const Vec& b) { return a.x * b.x + a.y * b.y; }
static inline double vcross(const Vec& a, const Vec& b) { return a.x * b.y - a.y * b.x; }

struct HP {
    Vec p, dir;
    double ang;
};

static inline bool hp_out(const HP& h, const Vec& pt, double eps) {
    return vcross(h.dir, pt - h.p) < -eps;
}

static inline Vec hp_cross(const HP& a, const HP& b) {
    double den = vcross(a.dir, b.dir);
    return a.p + a.dir * (vcross(b.p - a.p, b.dir) / den);
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    const double eps = 1e-9;
    vector<HP> norm;
    norm.reserve(n);
    for (int i = 0; i < n; ++i) {
        Vec p, dir;
        cin >> p.x >> p.y >> dir.x >> dir.y;
        double l = sqrt(dir.x * dir.x + dir.y * dir.y);
        if (l <= eps) continue;
        Vec d = dir / l;
        norm.push_back({p, d, atan2(d.y, d.x)});
    }
    vector<Vec> poly;
    if (norm.size() >= 3) {
        sort(norm.begin(), norm.end(), [](const HP& a, const HP& b) { return a.ang < b.ang; });
        vector<HP> uniq;
        uniq.reserve(norm.size());
        for (const auto& h : norm) {
            if (!uniq.empty()) {
                const auto& prev = uniq.back();
                if (fabs(vcross(prev.dir, h.dir)) <= eps && vdot(prev.dir, h.dir) > 0.0) {
                    if (vcross(prev.dir, h.p - prev.p) > 0.0) uniq.back() = h;
                    continue;
                }
            }
            uniq.push_back(h);
        }
        if (uniq.size() >= 2) {
            const auto& first = uniq.front();
            const auto& last = uniq.back();
            if (fabs(vcross(first.dir, last.dir)) <= eps && vdot(first.dir, last.dir) > 0.0) {
                if (vcross(first.dir, last.p - first.p) > 0.0) uniq[0] = last;
                uniq.pop_back();
            }
        }
        if (uniq.size() >= 3) {
            vector<HP> dq;
            dq.reserve(uniq.size());
            size_t head = 0;
            bool empty = false;
            for (const auto& h : uniq) {
                while (dq.size() - head >= 2 && hp_out(h, hp_cross(dq[dq.size() - 2], dq.back()), eps)) {
                    dq.pop_back();
                }
                while (dq.size() - head >= 2 && hp_out(h, hp_cross(dq[head], dq[head + 1]), eps)) {
                    head++;
                }
                if (dq.size() > head) {
                    const auto& prev = dq.back();
                    double c = vcross(prev.dir, h.dir);
                    if (fabs(c) <= eps) {
                        if (vdot(prev.dir, h.dir) < 0.0 && vcross(prev.dir, h.p - prev.p) < -eps) {
                            empty = true;
                            break;
                        }
                        continue;
                    }
                    if (c < -eps) {
                        empty = true;
                        break;
                    }
                }
                dq.push_back(h);
            }
            if (!empty) {
                while (dq.size() - head >= 3 && hp_out(dq[head], hp_cross(dq[dq.size() - 2], dq.back()), eps)) {
                    dq.pop_back();
                }
                while (dq.size() - head >= 3 && hp_out(dq.back(), hp_cross(dq[head], dq[head + 1]), eps)) {
                    head++;
                }
                size_t m = dq.size() - head;
                if (m >= 3) {
                    bool ok = true;
                    for (size_t i = 0; i < m; ++i) {
                        if (vcross(dq[head + i].dir, dq[head + (i + 1) % m].dir) <= eps) {
                            ok = false;
                            break;
                        }
                    }
                    if (ok) {
                        poly.reserve(m);
                        for (size_t i = 0; i < m; ++i) {
                            poly.push_back(hp_cross(dq[head + i], dq[head + (i + 1) % m]));
                        }
                    }
                }
            }
        }
    }
    double area = 0.0;
    for (size_t i = 0; i < poly.size(); ++i) {
        area += vcross(poly[i], poly[(i + 1) % poly.size()]);
    }
    area *= 0.5;
    cout << poly.size() << ' ' << (int64_t)llround(area * 1000.0) << '\n';
    return 0;
}
