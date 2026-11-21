#include <algorithm>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>
using namespace std;

struct Pt3 { int64_t x, y, z; };

static inline __int128 face_side(const Pt3& a, const Pt3& b, const Pt3& c, const Pt3& p) {
    __int128 ux = b.x - a.x, uy = b.y - a.y, uz = b.z - a.z;
    __int128 vx = c.x - a.x, vy = c.y - a.y, vz = c.z - a.z;
    __int128 nx = uy * vz - uz * vy;
    __int128 ny = uz * vx - ux * vz;
    __int128 nz = ux * vy - uy * vx;
    return nx * (p.x - a.x) + ny * (p.y - a.y) + nz * (p.z - a.z);
}

static inline bool col3(const Pt3& a, const Pt3& b, const Pt3& c) {
    __int128 ux = b.x - a.x, uy = b.y - a.y, uz = b.z - a.z;
    __int128 vx = c.x - a.x, vy = c.y - a.y, vz = c.z - a.z;
    return uy * vz == uz * vy && uz * vx == ux * vz && ux * vy == uy * vx;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<Pt3> pts(n);
    for (int i = 0; i < n; ++i) cin >> pts[i].x >> pts[i].y >> pts[i].z;

    int i0 = 0, i1 = -1, i2 = -1, i3 = -1;
    for (int i = 1; i < n; ++i) {
        if (pts[i].x != pts[i0].x || pts[i].y != pts[i0].y || pts[i].z != pts[i0].z) { i1 = i; break; }
    }
    if (i1 != -1) {
        for (int i = i1 + 1; i < n; ++i) {
            if (!col3(pts[i0], pts[i1], pts[i])) { i2 = i; break; }
        }
    }
    if (i2 != -1) {
        for (int i = i2 + 1; i < n; ++i) {
            if (face_side(pts[i0], pts[i1], pts[i2], pts[i]) != 0) { i3 = i; break; }
        }
    }
    if (i3 == -1) { cout << 0 << '\n'; return 0; }
    if (face_side(pts[i0], pts[i1], pts[i2], pts[i3]) > 0) swap(i1, i2);

    vector<tuple<int, int, int>> faces = {
        {i0, i1, i2}, {i0, i3, i1}, {i1, i3, i2}, {i2, i3, i0}
    };
    vector<tuple<int, int, int>> kept;
    vector<tuple<int64_t, int, int>> edges;
    for (int p = 0; p < n; ++p) {
        if (p == i0 || p == i1 || p == i2 || p == i3) continue;
        const Pt3& pp = pts[p];
        bool any_vis = false;
        for (auto [a, b, c] : faces) {
            if (face_side(pts[a], pts[b], pts[c], pp) > 0) {
                any_vis = true;
                break;
            }
        }
        if (!any_vis) continue;
        kept.clear();
        edges.clear();
        for (auto [a, b, c] : faces) {
            if (face_side(pts[a], pts[b], pts[c], pp) > 0) {
                edges.push_back({(int64_t)min(a, b) * n + max(a, b), a, b});
                edges.push_back({(int64_t)min(b, c) * n + max(b, c), b, c});
                edges.push_back({(int64_t)min(c, a) * n + max(c, a), c, a});
            } else {
                kept.push_back({a, b, c});
            }
        }
        sort(edges.begin(), edges.end());
        for (size_t ei = 0; ei < edges.size(); ) {
            if (ei + 1 < edges.size() && get<0>(edges[ei]) == get<0>(edges[ei + 1])) {
                ei += 2;
            } else {
                kept.push_back({get<1>(edges[ei]), get<2>(edges[ei]), p});
                ei += 1;
            }
        }
        faces.swap(kept);
    }
    cout << faces.size() << '\n';
    return 0;
}
