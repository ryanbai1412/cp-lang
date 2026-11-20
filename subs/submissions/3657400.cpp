#include <bits/stdc++.h>

using namespace std;

#define long int64_t

struct ST {
    int n;
    vector<long> v, lazy, arr;
    ST(int n, const vector<long> &arr): n(n), v(4 * n), lazy(4 * n), arr(arr) {build(1, 0, n - 1);}
    void build(int o, int l, int r) {
        if(l == r) v[o] = arr[l];
        else {
            int mid = (l + r) / 2;
            build(o * 2, l, mid);
            build(o * 2 + 1, mid + 1, r);
            v[o] = min(v[o * 2], v[o * 2 + 1]);
        }
    }
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
    vector<long> arr(n);
    for(int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    ST st(n, arr);
    while(q--) {
        int l, r, x;
        cin >> l >> r >> x;
        l--;
        r--;
        st.update(l, r, -x);
        cout << st.query(l, r) << " " << st.query(0, n - 1) << "\n";
    }
}