#include "bits/extc++.h"
 
using namespace std;
 
template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}
 
template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}
 
#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif
 
using ll = long long;
 
#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())
 
const int maxn = 1 << 17;
 
struct ST {
    int n;
    vector<vector<int>> v;
 
    ST(const vector<int>& arr) : n(sz(arr)), v(__lg(n) + 3, vector<int>(n)) {
        v[0] = arr;
        for (int i = 1; (1 << i) <= n; i++) {
            for (int j = 0; j + (1 << i) <= n; j++) {
                v[i][j] = max(v[i - 1][j], v[i - 1][j + (1 << i)]);
            }
        }
    }
 
    int query(int l, int r) const {
        int k = __lg(r - l + 1);
        return max(v[k][l], v[k][r - (1 << k) + 1]);
    }
};
 
struct Node {
    int mx;
    long pref[21], suff[21], vl[21], vr[21];
 
    friend Node operator+(const Node& a, const Node& b) {
        Node ans {};
        ans.mx = max(a.mx, b.mx);
        for (int i = 0; i <= 20; i++) {
            ans.pref[i] = a.pref[i] + b.pref[max(a.mx, i)];
            ans.suff[i] = b.suff[i] + a.suff[max(b.mx, i)];
        }
        memset(ans.vl, 0x3f, sizeof(ans.vl));
        memset(ans.vr, 0x3f, sizeof(ans.vr));
        auto upd = [&](int l, int r, long x) -> void {
            if (l == ans.mx) {
                ans.vr[r] = min(ans.vr[r], x);
            }
            if (r == ans.mx) {
                ans.vl[l] = min(ans.vl[l], x);
            }
        };
        for (int i = 0; i <= 20; i++) {
            upd(i, ans.mx, a.vl[i] + b.pref[a.mx]);
            upd(a.mx, max(i, b.mx), a.vr[i] + b.pref[i]);
        }
        for (int i = 0; i <= 20; i++) {
            upd(max(a.mx, i), b.mx, b.vl[i] + a.suff[i]);
            upd(ans.mx, i, b.vr[i] + a.suff[b.mx]);
        }
        return ans;
    }
 
    long solve() const {
        return min(*min_element(begin(vl), end(vl)),
                   *min_element(begin(vr), end(vr)));
    }
 
    static Node from(int x) {
        Node n {};
        n.mx = x;
        for (int i = 0; i <= 20; i++) {
            n.pref[i] = n.suff[i] = max(i, x);
        }
        memset(n.vl, 0x3f, sizeof(n.vl));
        memset(n.vr, 0x3f, sizeof(n.vr));
        n.vl[x] = n.vr[x] = x;
        return n;
    }
} v[maxn << 1];
 
int n, q;
vector<int> arr;
 
void build(int o, int l, int r) {
    if (l == r) {
        v[o] = Node::from(arr[l]);
        return;
    }
    int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
    build(lc, l, mid);
    build(rc, mid + 1, r);
    v[o] = v[lc] + v[rc];
}
 
Node query(int o, int l, int r, int ql, int qr) {
    if (ql <= l && r <= qr) {
        return v[o];
    }
    int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
    if (ql <= mid) {
        if (mid < qr) {
            return query(lc, l, mid, ql, qr) + query(rc, mid + 1, r, ql, qr);
        }
        return query(lc, l, mid, ql, qr);
    }
    return query(rc, mid + 1, r, ql, qr);
}
 
vector<ll> minimum_costs(vector<int> _arr, vector<int> ql, vector<int> qr) {
    arr = _arr;
    n = sz(arr);
    q = sz(ql);
    vector<ll> qans;
    if (n <= 5000 && q <= 5000) {
        long pref[n][n], suff[n][n];
        for (int i = 0; i < n; i++) {
            int cmx = arr[i];
            pref[i][i] = cmx;
            for (int j = i - 1; j >= 0; j--) {
                pref[i][j] = pref[i][j + 1] + (cmx = max(cmx, arr[j]));
            }
        }
        for (int i = 0; i < n; i++) {
            int cmx = arr[i];
            suff[i][i] = cmx;
            for (int j = i + 1; j < n; j++) {
                suff[i][j] = suff[i][j - 1] + (cmx = max(cmx, arr[j]));
            }
        }
        for (int qi = 0; qi < q; qi++) {
            int l = ql[qi], r = qr[qi];
            long ans = 1e18;
            for (int i = l; i <= r; i++) {
                ans = min(ans, pref[i][l] + suff[i][r] - arr[i]);
            }
            qans.push_back(ans);
        }
    } else {
        build(1, 0, n - 1);
        for (int qi = 0; qi < q; qi++) {
            int l = ql[qi], r = qr[qi];
            qans.push_back(query(1, 0, n - 1, l, r).solve());
        }
    }
    return qans;
}