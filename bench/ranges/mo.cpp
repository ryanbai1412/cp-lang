#include <algorithm>
#include <cmath>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <vector>
using namespace std;
using ll = long long;

int main() {
    const int n = 100000, q = 100000, vmax = 100000;
    ll seed = 123456789;
    auto next_rng = [&]() -> ll {
        seed = (seed * 1103515245LL + 12345LL) % 2147483648LL;
        return seed;
    };
    vector<int> a(n);
    for (int i = 0; i < n; i++) a[i] = (int)(next_rng() % vmax);
    vector<int> ql(q), qr(q);
    for (int i = 0; i < q; i++) {
        int x = (int)(next_rng() % n);
        int y = (int)(next_rng() % n);
        ql[i] = min(x, y);
        qr[i] = max(x, y) + 1;
    }
    int b = max(1, n / max(1, (int)sqrt((double)q)));
    vector<int> order(q);
    iota(order.begin(), order.end(), 0);
    sort(order.begin(), order.end(), [&](int i, int j) {
        int bi = ql[i] / b, bj = ql[j] / b;
        if (bi != bj) return bi < bj;
        return (bi & 1) ? (qr[i] > qr[j]) : (qr[i] < qr[j]);
    });
    vector<int> freq(vmax, 0);
    int distinct = 0;
    auto add_pos = [&](int p) {
        int v = a[p];
        if (freq[v] == 0) distinct++;
        freq[v]++;
    };
    auto rem_pos = [&](int p) {
        int v = a[p];
        freq[v]--;
        if (freq[v] == 0) distinct--;
    };
    vector<ll> ans(q);
    int l = 0, r = 0;
    for (int qi : order) {
        int nl = ql[qi], nr = qr[qi];
        while (l > nl) add_pos(--l);
        while (r < nr) add_pos(r++);
        while (l < nl) rem_pos(l++);
        while (r > nr) rem_pos(--r);
        ans[qi] = distinct;
    }
    ll acc = 0;
    for (int i = 0; i < q; i++) acc += (ll)(i + 1) * ans[i];
    cout << acc << '\n';
}
