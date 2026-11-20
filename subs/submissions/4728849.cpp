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

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

struct ST {
    int n;
    vector<int> v, arr;

    ST(const vector<int>& arr) : n(sz(arr)), v(4 * n), arr(arr) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = arr[l];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ind, int x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        }
        if (mid < ind) {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = v[lc] + v[rc];
    }

    void update(int ind, int x) {
        dbg(ind, x);
        update(1, 0, n - 1, ind, x);
    }

    int query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1, ans = 0;
        if (ql <= mid) {
            ans += query(lc, l, mid, ql, qr);
        }
        if (mid < qr) {
            ans += query(rc, mid + 1, r, ql, qr);
        }
        return ans;
    }

    int query(int l, int r) const {
        if (l > r) {
            return 0;
        }
        return query(1, 0, n - 1, l, r);
    }

    pair<int, int> bsearch(int x) {
        int l = -1, r = n - 1;
        while (l < r) {
            int mid = (l + r + 1) / 2;
            if (query(0, mid) <= x) {
                l = mid;
            } else {
                r = mid - 1;
            }
        }
        return {query(0, l), l + 1};
    }
};

vector<int> nxta(const vector<int>& arr) {
    int n = sz(arr);
    vector<int> ans(n);
    merge(arr.begin(), arr.begin() + n / 2, arr.begin() + n / 2, arr.end(),
          ans.begin());
    return ans;
}

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int iarr[n];
    for (int i = 0; i < n; i++) {
        iarr[arr[i]] = i;
    }
    int nxt[n];
    {
        vector<int> st;
        for (int i = n - 1; i >= 0; i--) {
            while (sz(st) && arr[st.back()] <= arr[i]) {
                st.pop_back();
            }
            if (sz(st)) {
                nxt[i] = st.back();
            } else {
                nxt[i] = n;
            }
            st.push_back(i);
        }
    }
    map<int, int> segs;
    vector<int> starr(n);
    for (int i = 0; i < n; i = nxt[i]) {
        segs[arr[i]] = starr[arr[i]] = nxt[i] - i;
    }
    ST st(starr);
    dbg(starr);
    int ans[q];
    vector<pair<int, int>> queries[n + 1];
    for (int i = 0; i < q; i++) {
        int t, ind;
        cin >> t >> ind;
        ind--;
        queries[min(n, t)].emplace_back(ind, i);
    }
    for (int i = 0; i <= n; i++) {
        for (auto& [ind, qi] : queries[i]) {
            auto [pref, j] = st.bsearch(ind);
            dbg(j, st.query(0, j - 1), st.query(0, j), iarr[j], pref, ind);
            ans[qi] = arr[iarr[j] + ind - pref];
        }
        auto [pref, j] = st.bsearch(n / 2);
        if (pref != n / 2) {
            auto it = segs.find(j);
            int len = it->second;
            st.update(j, it->second = n / 2 - pref);
            dbg(iarr[j], pref, iarr[j] + it->second, pref + len);
            for (int k = iarr[j] + it->second; k < iarr[j] + len; k = nxt[k]) {
                st.update(arr[k],
                          segs[arr[k]] = min(iarr[j] + len, nxt[k]) - k);
            }
        }
        dbg(i);
    }
    for (auto& a : ans) {
        cout << a + 1 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}