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
#define sz(x) int(size(x))

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
        v[o] = max(v[lc], v[rc]) + lazy[o];
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    int query() const {
        return v[1];
    }
};

void solve() {
    int n, m;
    cin >> n >> m;
    array<int, 3> arr[n];
    for (auto& [l, r, len] : arr) {
        cin >> l >> r;
        len = r - l;
    }
    vector<int> comp;
    for (auto& [l, r, _] : arr) {
        comp.push_back(l);
        comp.push_back(r);
    }
    sort(begin(comp), end(comp));
    comp.erase(unique(begin(comp), end(comp)), comp.end());
    for (auto& [l, r, _] : arr) {
        for (auto* b : {&l, &r}) {
            *b = int(lower_bound(begin(comp), end(comp), *b) - comp.begin());
        }
    }
    sort(arr, arr + n, [&](const auto& a, const auto& b) -> bool {
        return a[2] < b[2];
    });
    auto check = [&](int x) -> bool {
        ST st(sz(comp));
        for (int i = 0, j = 0; i < n; i++) {
            while (j < n && arr[j][2] - arr[i][2] <= x) {
                st.update(arr[j][0], arr[j][1], +1);
                j++;
            }
            if (st.query() >= m) {
                return true;
            }
            st.update(arr[i][0], arr[i][1], -1);
        }
        return false;
    };
    int l = 0, r = 1e9 + 1;
    while (l < r) {
        int mid = (l + r) / 2;
        if (check(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    if (l > 1e9) {
        cout << -1 << endl;
    } else {
        cout << l << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}