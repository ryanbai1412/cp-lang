#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;
using ll = long long;

struct RangeKth {
    vector<ll> vals;
    vector<int> lc, rc, cnt, roots;
    int m;
    int update(int prev, int l, int r, int pos) {
        int id = (int)lc.size();
        lc.push_back(lc[prev]);
        rc.push_back(rc[prev]);
        cnt.push_back(cnt[prev] + 1);
        if (r - l > 1) {
            int mid = (l + r) / 2;
            if (pos < mid) { int c = update(lc[prev], l, mid, pos); lc[id] = c; }
            else { int c = update(rc[prev], mid, r, pos); rc[id] = c; }
        }
        return id;
    }
    RangeKth(const vector<ll>& a) : vals(a) {
        sort(vals.begin(), vals.end());
        vals.erase(unique(vals.begin(), vals.end()), vals.end());
        m = (int)vals.size();
        int cap = (int)a.size() * 20 + 4;
        lc.reserve(cap); rc.reserve(cap); cnt.reserve(cap);
        lc.push_back(0); rc.push_back(0); cnt.push_back(0);
        roots.resize(a.size() + 1, 0);
        for (size_t i = 0; i < a.size(); i++) {
            int pos = (int)(lower_bound(vals.begin(), vals.end(), a[i]) - vals.begin());
            roots[i + 1] = update(roots[i], 0, m, pos);
        }
    }
    ll kth(int l_idx, int r_idx, int k) const {
        int u = roots[l_idx], v = roots[r_idx], l = 0, r = m;
        while (r - l > 1) {
            int mid = (l + r) / 2;
            int left_cnt = cnt[lc[v]] - cnt[lc[u]];
            if (k < left_cnt) { u = lc[u]; v = lc[v]; r = mid; }
            else { k -= left_cnt; u = rc[u]; v = rc[v]; l = mid; }
        }
        return vals[l];
    }
};

int main() {
    const int n = 50000, q = 100000;
    ll seed = 123456789;
    auto next_rng = [&]() -> ll {
        seed = (seed * 1103515245LL + 12345LL) % 2147483648LL;
        return seed;
    };
    vector<ll> a(n);
    for (int i = 0; i < n; i++) a[i] = next_rng() % 1000000;
    RangeKth rk(a);
    ll acc = 0;
    for (int i = 0; i < q; i++) {
        int x = (int)(next_rng() % n), y = (int)(next_rng() % n);
        int lo = min(x, y), hi = max(x, y);
        acc += rk.kth(lo, hi + 1, (hi - lo) / 2);
    }
    cout << acc << "\n";
}
