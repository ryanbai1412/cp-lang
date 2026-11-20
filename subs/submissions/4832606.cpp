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
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
    tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

struct ST {
    int n;
    vector<int> v, lazy;

    ST(int n) : n(n), v(4 * n), lazy(4 * n) {}

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            v[o] += x;
            lazy[o] += x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = min(v[lc], v[rc]) + lazy[o];
    }

    void update(int l, int r, int x) {
        if (l >= r) {
            return;
        }
        update(1, 0, n - 1, l, r - 1, x);
    }

    int query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1, ans = 1e9;
        if (ql <= mid) {
            ans = min(ans, query(lc, l, mid, ql, qr));
        }
        if (mid < qr) {
            ans = min(ans, query(rc, mid + 1, r, ql, qr));
        }
        return ans + lazy[o];
    }
    
    int query(int l, int r) const {
        if (l >= r) {
            return 0;
        }
        return query(1, 0, n - 1, l, r - 1);
    }
};

struct PSA {
    vector<int> psum;

    PSA(const vector<int>& arr) {
        psum.push_back(0);
        for (auto& a : arr) {
            psum.push_back(psum.back() + a);
        }
    }

    int query(int l, int r) const {
        if (l >= r) {
            return 0;
        }
        return psum[r] - psum[l];
    }
};

int coord_comp(vector<int*> arr) {
    sort(begin(arr), end(arr), [&](const int* a, const int* b) -> bool {
        return *a < *b;
    });
    int j = 0, prv;
    for (int i = 0; i < sz(arr); i++) {
        j += i && prv != *arr[i];
        prv = *arr[i];
        *arr[i] = j;
    }
    return j + 1;
}

void solve() {
    int n, m;
    cin >> n >> m;
    int conds[m][3];
    for (auto& [l, r, x] : conds) {
        cin >> l >> r >> x;
        l--;
        r--;
    }
    {
        vector<int*> comp;
        for (auto& a : conds) {
            comp.push_back(&a[2]);
        }
        m = coord_comp(comp);
    }
    int arr[n] {};
    bool eqs[n] {};
    for (auto& [l, r, x] : conds) {
        fill(arr + l, arr + r + 1, x);
        eqs[l] = true;
    }
    long ans = 0;
    {
        ost<pair<int, int>> s;
        for (int i = n - 1; i >= 0; i--) {
            if (eqs[i]) {
                ans += long(s.order_of_key({arr[i], -1}));
                s.insert({arr[i], i});
            }
        }
    }
    ST st(m);
    for (int i = 0; i < n; i++) {
        if (eqs[i]) {
            st.update(arr[i] + 1, n, 1);
        }
    }
    for (int i = 0; i < n; i++) {
        st.update(0, arr[i], 1);
        if (eqs[i]) {
            st.update(arr[i] + 1, n, -1);
        } else {
            ans += st.query(arr[i], m);
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}