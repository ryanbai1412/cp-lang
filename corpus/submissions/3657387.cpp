#include <bits/stdc++.h>

using namespace std;

#define long int64_t

struct ST {
    int n;
    vector<long> v, lazy;
    ST(int n): n(n), v(4 * n), lazy(4 * n) {}
    void update(int o, int l, int r, int ql, int qr, long x) {
        if(ql <= l && r <= qr) v[o] += x, lazy[o] += x;
        else {
            int mid = (l + r) / 2;
            if(ql <= mid) update(o * 2, l, mid, ql, qr, x);
            if(mid < qr) update(o * 2 + 1, mid + 1, r, ql, qr, x);
            v[o] = min(v[o * 2], v[o * 2 + 1]);
        }
    }
    void update(int l, int r, long x) {
        update(1, 0, n - 1, l, r, x);
    }
    long query(int o, int l, int r, int ql, int qr, long clazy) {
        if(ql <= l && r <= qr) return v[o] + clazy;
        else {
            int mid = (l + r) / 2;
            long ans = 1e18;
            clazy += lazy[o];
            if(ql <= mid) ans = min(ans, query(o * 2, l, mid, ql, qr, clazy));
            if(mid < qr) ans = min(ans, query(o * 2 + 1, mid + 1, r, ql, qr, clazy));
            return ans;
        }
    }
    long query(int l, int r) {
        return max(INT64_C(0), query(1, 0, n - 1, l, r, 0));
    }
};

int main() {
    int n, q;
    cin >> n >> q;
    ST st(n);
    for(int i = 0; i < n; i++) {
        int x;
        cin >> x;
        st.update(i, i, x);
    }
    while(q--) {
        int l, r, x;
        cin >> l >> r >> x;
        l--;
        r--;
        st.update(l, r, -x);
        cout << st.query(l, r) << " " << st.query(0, n - 1) << "\n";
    }
}