#include <algorithm>
#include <cstdint>
#include <iostream>
#include <limits>
#include <vector>
using namespace std;
using ll = long long;
static constexpr ll INF = numeric_limits<ll>::max() / 4;

struct Beats {
    int n;
    vector<ll> sum, mx, smx, mn, smn, add;
    vector<int> mxc, mnc, len;
    Beats(const vector<ll>& a) : n((int)a.size()),
        sum(4 * n), mx(4 * n), smx(4 * n), mn(4 * n), smn(4 * n), add(4 * n),
        mxc(4 * n), mnc(4 * n), len(4 * n) {
        build(1, 0, n, a);
    }
    void pull(int u) {
        int l = 2 * u, r = 2 * u + 1;
        sum[u] = sum[l] + sum[r];
        if (mx[l] == mx[r]) { mx[u] = mx[l]; mxc[u] = mxc[l] + mxc[r]; smx[u] = max(smx[l], smx[r]); }
        else if (mx[l] > mx[r]) { mx[u] = mx[l]; mxc[u] = mxc[l]; smx[u] = max(smx[l], mx[r]); }
        else { mx[u] = mx[r]; mxc[u] = mxc[r]; smx[u] = max(mx[l], smx[r]); }
        if (mn[l] == mn[r]) { mn[u] = mn[l]; mnc[u] = mnc[l] + mnc[r]; smn[u] = min(smn[l], smn[r]); }
        else if (mn[l] < mn[r]) { mn[u] = mn[l]; mnc[u] = mnc[l]; smn[u] = min(smn[l], mn[r]); }
        else { mn[u] = mn[r]; mnc[u] = mnc[r]; smn[u] = min(mn[l], smn[r]); }
    }
    void build(int u, int l, int r, const vector<ll>& a) {
        len[u] = r - l;
        if (r - l == 1) {
            sum[u] = mx[u] = mn[u] = a[l];
            smx[u] = -INF; smn[u] = INF;
            mxc[u] = mnc[u] = 1;
            return;
        }
        int m = (l + r) / 2;
        build(2 * u, l, m, a);
        build(2 * u + 1, m, r, a);
        pull(u);
    }
    void push_add(int u, ll v) {
        sum[u] += v * len[u];
        mx[u] += v; if (smx[u] > -INF) smx[u] += v;
        mn[u] += v; if (smn[u] < INF) smn[u] += v;
        add[u] += v;
    }
    void push_min(int u, ll v) {
        if (v >= mx[u]) return;
        sum[u] += (v - mx[u]) * mxc[u];
        if (mn[u] == mx[u]) mn[u] = v;
        else if (smn[u] == mx[u]) smn[u] = v;
        mx[u] = v;
    }
    void push_max(int u, ll v) {
        if (v <= mn[u]) return;
        sum[u] += (v - mn[u]) * mnc[u];
        if (mx[u] == mn[u]) mx[u] = v;
        else if (smx[u] == mn[u]) smx[u] = v;
        mn[u] = v;
    }
    void push(int u) {
        int l = 2 * u, r = 2 * u + 1;
        if (add[u] != 0) { push_add(l, add[u]); push_add(r, add[u]); add[u] = 0; }
        push_min(l, mx[u]); push_min(r, mx[u]);
        push_max(l, mn[u]); push_max(r, mn[u]);
    }
    void chmin(int u, int l, int r, int ql, int qr, ll v) {
        if (qr <= l || r <= ql || v >= mx[u]) return;
        if (ql <= l && r <= qr && v > smx[u]) { push_min(u, v); return; }
        push(u); int m = (l + r) / 2;
        chmin(2 * u, l, m, ql, qr, v); chmin(2 * u + 1, m, r, ql, qr, v);
        pull(u);
    }
    void chmax(int u, int l, int r, int ql, int qr, ll v) {
        if (qr <= l || r <= ql || v <= mn[u]) return;
        if (ql <= l && r <= qr && v < smn[u]) { push_max(u, v); return; }
        push(u); int m = (l + r) / 2;
        chmax(2 * u, l, m, ql, qr, v); chmax(2 * u + 1, m, r, ql, qr, v);
        pull(u);
    }
    void range_add(int u, int l, int r, int ql, int qr, ll v) {
        if (qr <= l || r <= ql) return;
        if (ql <= l && r <= qr) { push_add(u, v); return; }
        push(u); int m = (l + r) / 2;
        range_add(2 * u, l, m, ql, qr, v); range_add(2 * u + 1, m, r, ql, qr, v);
        pull(u);
    }
    ll range_sum(int u, int l, int r, int ql, int qr) {
        if (qr <= l || r <= ql) return 0;
        if (ql <= l && r <= qr) return sum[u];
        push(u); int m = (l + r) / 2;
        return range_sum(2 * u, l, m, ql, qr) + range_sum(2 * u + 1, m, r, ql, qr);
    }
};

int main() {
    const int n = 50000, q = 100000;
    ll seed = 123456789;
    auto next_rng = [&]() -> ll {
        seed = (seed * 1103515245LL + 12345LL) % 2147483648LL;
        return seed;
    };
    vector<ll> init(n);
    for (int i = 0; i < n; i++) init[i] = (next_rng() % 200001) - 100000;
    Beats seg(init);
    ll acc = 0;
    for (int step = 0; step < q; step++) {
        int a = (int)(next_rng() % n), b = (int)(next_rng() % n);
        int lo = min(a, b), hi = max(a, b) + 1;
        int op = step % 4;
        if (op == 0) seg.chmin(1, 0, n, lo, hi, (next_rng() % 200001) - 100000);
        else if (op == 1) seg.chmax(1, 0, n, lo, hi, (next_rng() % 200001) - 100000);
        else if (op == 2) seg.range_add(1, 0, n, lo, hi, (next_rng() % 201) - 100);
        else acc += seg.range_sum(1, 0, n, lo, hi);
    }
    cout << acc << "\n";
}
