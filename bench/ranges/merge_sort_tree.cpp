#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;
using ll = long long;

struct MergeSortTree {
    int size;
    vector<vector<ll>> nodes;
    MergeSortTree(const vector<ll>& a) {
        size = 1;
        while (size < (int)a.size()) size <<= 1;
        nodes.resize(2 * size);
        for (size_t i = 0; i < a.size(); i++) nodes[size + i] = {a[i]};
        for (int p = size - 1; p >= 1; p--) {
            const auto& l = nodes[2 * p];
            const auto& r = nodes[2 * p + 1];
            nodes[p].resize(l.size() + r.size());
            merge(l.begin(), l.end(), r.begin(), r.end(), nodes[p].begin());
        }
    }
    int count_le(int l, int r, ll x) const {
        int res = 0;
        for (l += size, r += size; l < r; l >>= 1, r >>= 1) {
            if (l & 1) {
                res += (int)(upper_bound(nodes[l].begin(), nodes[l].end(), x) - nodes[l].begin());
                l++;
            }
            if (r & 1) {
                --r;
                res += (int)(upper_bound(nodes[r].begin(), nodes[r].end(), x) - nodes[r].begin());
            }
        }
        return res;
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
    MergeSortTree mst(a);
    ll acc = 0;
    for (int i = 0; i < q; i++) {
        int x = (int)(next_rng() % n), y = (int)(next_rng() % n);
        int lo = min(x, y), hi = max(x, y);
        ll v = next_rng() % 1000000;
        acc += mst.count_le(lo, hi + 1, v);
    }
    cout << acc << "\n";
}
