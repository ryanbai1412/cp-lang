// COCI 2020/2021 Round 6 #4 (Geometrija). An uncrossed segment lies in every triangulation, so
// only the <= 3n edges of one sweep triangulation are candidates. A candidate AB is crossed iff
// some C left of AB and D right of it have angle(C at A) + angle(D at A) < pi and the same at
// B: sorting the left points by their angle at A makes the C's that pair with a D a prefix,
// and the prefix's smallest angle at B settles it, O(n log n) per candidate.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
typedef __int128 L;
struct P { ll x, y; };
P operator-(P a, P b) { return {a.x - b.x, a.y - b.y}; }
L cross(P a, P b) { return (L)a.x * b.y - (L)a.y * b.x; }
L ccw(P a, P b, P c) { return cross(b - a, c - a); }
int n;
vector<P> p;
// Angle at o from ray o->t, for points q on one fixed side: smaller means closer to the ray.
bool crossed(int A, int B) {
    vector<int> left, right;
    for (int i = 0; i < n; i++) {
        if (i == A || i == B) continue;
        (ccw(p[A], p[B], p[i]) > 0 ? left : right).push_back(i);
    }
    if (left.empty() || right.empty()) return false;
    P a = p[A], b = p[B];
    // Left points by angle at A from AB: q before r iff r is left of ray A->q.
    sort(left.begin(), left.end(), [&](int q, int r) { return ccw(a, p[q], p[r]) > 0; });
    // Prefix minimum of the angle at B from BA (left of AB is right of BA).
    vector<int> best(left.size());
    for (size_t i = 0; i < left.size(); i++) {
        best[i] = left[i];
        if (i && ccw(b, p[best[i - 1]], p[left[i]]) < 0) best[i] = best[i - 1];
    }
    for (int d : right) {
        // C pairs with D at A iff ccw(A, D, C) > 0; true on a prefix of `left`.
        int lo = 0, hi = left.size();
        while (lo < hi) {
            int mid = (lo + hi) / 2;
            if (ccw(a, p[d], p[left[mid]]) > 0) lo = mid + 1; else hi = mid;
        }
        if (lo == 0) continue;
        if (ccw(b, p[best[lo - 1]], p[d]) > 0) return true;
    }
    return false;
}
int main() {
    scanf("%d", &n);
    p.resize(n);
    for (auto& q : p) scanf("%lld %lld", &q.x, &q.y);
    vector<int> ord(n);
    iota(ord.begin(), ord.end(), 0);
    sort(ord.begin(), ord.end(), [&](int i, int j) { return p[i].x != p[j].x ? p[i].x < p[j].x : p[i].y < p[j].y; });
    set<pair<int, int>> edges;
    auto add = [&](int i, int j) { edges.insert({min(i, j), max(i, j)}); };
    vector<int> up, lo;
    for (int i : ord) {
        if (!up.empty()) add(up.back(), i);
        while (up.size() >= 2 && ccw(p[up[up.size() - 2]], p[up.back()], p[i]) >= 0) up.pop_back(), add(up.back(), i);
        while (lo.size() >= 2 && ccw(p[lo[lo.size() - 2]], p[lo.back()], p[i]) <= 0) lo.pop_back(), add(lo.back(), i);
        up.push_back(i), lo.push_back(i);
    }
    int ans = 0;
    for (auto [i, j] : edges) ans += !crossed(i, j);
    printf("%d\n", ans);
}
