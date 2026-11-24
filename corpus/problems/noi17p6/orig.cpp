// noi17p6: the hull of the survivors at each time. The points are peeled into convex layers
// L_1, L_2, ... once. The survivors of L_1 stay hull vertices; between two consecutive survivors
// a, b of L_1 the hull is that of a, b and the survivors strictly right of a->b, and those can
// only come from the arcs of L_2, L_3, ... right of a->b, down to the first arc that is empty or
// lost nothing: that arc's polygon, a and b cover every deeper point right of a->b.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;

struct P { ll x, y; };
P operator-(P a, P b) { return {a.x - b.x, a.y - b.y}; }
ll cross(P a, P b) { return a.x * b.y - a.y * b.x; }
ll cross(P o, P a, P b) { return cross(a - o, b - o); }

// strict CCW hull of the given point ids, in the order given (sorted by x, y)
vector<int> hull(const vector<int>& ids, const vector<P>& p) {
    int m = ids.size();
    if (m < 3) return ids;
    vector<int> h(2 * m);
    int k = 0;
    for (int i = 0; i < m; i++) {
        while (k >= 2 && cross(p[h[k - 2]], p[h[k - 1]], p[ids[i]]) <= 0) k--;
        h[k++] = ids[i];
    }
    for (int i = m - 2, t = k + 1; i >= 0; i--) {
        while (k >= t && cross(p[h[k - 2]], p[h[k - 1]], p[ids[i]]) <= 0) k--;
        h[k++] = ids[i];
    }
    h.resize(k - 1);
    return h;
}

int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<P> p(n);
    for (auto& q : p) scanf("%lld %lld", &q.x, &q.y);
    vector<int> rest(n);
    iota(rest.begin(), rest.end(), 0);
    sort(rest.begin(), rest.end(), [&](int a, int b) { return p[a].x != p[b].x ? p[a].x < p[b].x : p[a].y < p[b].y; });
    vector<vector<int>> L;
    vector<int> layer(n), pos(n);
    vector<char> peeled(n);
    while (!rest.empty()) {
        vector<int> h = hull(rest, p);
        if (rest.size() >= 3 && h.size() < 3) h = rest;  // the rest are collinear
        for (size_t i = 0; i < h.size(); i++) layer[h[i]] = L.size(), pos[h[i]] = i, peeled[h[i]] = 1;
        L.push_back(h);
        vector<int> keep;
        for (int i : rest)
            if (!peeled[i]) keep.push_back(i);
        rest.swap(keep);
    }
    const vector<int>& O = L[0];
    int h0 = O.size();
    ll whole = 0;
    for (int i = 0; i < h0; i++) whole += cross(p[O[i]], p[O[(i + 1) % h0]]);
    vector<char> gone(n);
    ll S = -1;
    for (int t = 0; t < m; t++) {
        int k;
        scanf("%d", &k);
        vector<int> del(k);
        for (auto& d : del) {
            ll c;
            scanf("%lld", &c);
            d = (int)(((S + c) % n + n) % n);
            gone[d] = 1;
        }
        vector<int> outer;
        for (int d : del)
            if (layer[d] == 0) outer.push_back(pos[d]);
        sort(outer.begin(), outer.end());
        outer.erase(unique(outer.begin(), outer.end()), outer.end());
        ll area = whole;
        // a survivor of L_1 right before a deleted run
        int start = -1;
        for (int i = 0; i < h0 && start < 0; i++)
            if (!gone[O[i]] && gone[O[(i + 1) % h0]]) start = i;
        for (int s = 0; start >= 0 && s < h0;) {
            int ia = (start + s) % h0;
            if (gone[O[ia]] || !gone[O[(ia + 1) % h0]]) { s++; continue; }
            int len = 1;
            while (gone[O[(ia + len) % h0]]) len++;
            int ib = (ia + len) % h0;
            P a = p[O[ia]], b = p[O[ib]];
            for (int j = 0; j < len; j++) area -= cross(p[O[(ia + j) % h0]], p[O[(ia + j + 1) % h0]]);
            vector<int> cand = {O[ia], O[ib]};
            for (size_t j = 1; j < L.size(); j++) {
                bool any = false, lost = false;
                for (int v : L[j])
                    if (cross(a, b, p[v]) < 0) {
                        any = true;
                        if (gone[v]) lost = true;
                        else cand.push_back(v);
                    }
                if (!any || !lost) break;
            }
            sort(cand.begin(), cand.end(), [&](int u, int v) { return p[u].x != p[v].x ? p[u].x < p[v].x : p[u].y < p[v].y; });
            vector<int> hh = hull(cand, p);
            int r = find(hh.begin(), hh.end(), O[ia]) - hh.begin(), z = hh.size();
            for (int j = 0; hh[(r + j) % z] != O[ib]; j++) area += cross(p[hh[(r + j) % z]], p[hh[(r + j + 1) % z]]);
            s += len;
        }
        S = area;
        printf("%lld\n", S);
        for (int d : del) gone[d] = 0;
    }
}
