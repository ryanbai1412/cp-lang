#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxq = 1 << 19, maxa = 1 << 20;

struct ST1 {
    int v[maxa << 1];

    ST1() {
        fill(begin(v), end(v), -1e9);
    }

    void update(int o, int l, int r, int ind, int x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = max(v[lc], v[rc]);
    }

    void update(int ind, int x) {
        update(1, 0, maxa - 1, ind, x);
    }

    int query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return max(query(lc, l, mid, ql, qr),
                           query(rc, mid + 1, r, ql, qr));
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    int query(int l, int r) const {
        return query(1, 0, maxa - 1, l, r);
    }
};

struct DS1 {
    ST1 st;
    multiset<int> rcoords, arrl[maxa];
    pair<int, int> ans {int(1e9), 0};
    vector<pair<pair<int, int>, pair<int, int>>> hist;

    void insert(int l, int r) {
        rcoords.insert(r);
        arrl[r].insert(l);
        st.update(r, *(--arrl[r].end()));
        dbg(r);
        hist.push_back({{l, r}, ans});
        ans = min(ans, {r - l, r - l});
        {
            int cl = st.query(0, l);
            dbg(cl);
            if (cl >= 0) {
                ans = min(ans, {0, r - cl});
            }
        }
        {
            auto it = rcoords.lower_bound(l);
            if (it != rcoords.end()) {
                int nr = *it;
                assert(nr <= r);
                ans = min(ans, {nr - l, r - min(l, *(--arrl[nr].end()))});
            }
        }
    }

    pair<int, int> query() const {
        return ans;
        // pair<int, int> ans {int(1e9), 0};
        // for (int i = 0; i < sz(arr); i++) {
        //     auto& [l2, r2] = arr[i];
        //     ans = min(ans, {r2 - l2, r2 - l2});
        //     for (int j = i + 1; j < sz(arr); j++) {
        //         auto& [l1, r1] = arr[j];
        //         ans = min(ans, {max(0, r2 - l1), r1 - l2});
        //     }
        // }
        // return ans;
    }

    void rollback() {
        auto [q, qans] = hist.back();
        hist.pop_back();
        auto& [l, r] = q;
        ans = qans;
        rcoords.erase(rcoords.find(r));
        arrl[r].erase(arrl[r].find(l));
        if (!sz(arrl[r])) {
            st.update(r, -1e9);
        } else {
            st.update(r, *(--arrl[r].end()));
        }
    }
};

struct DS2 {
    DS1 dsf, dsr;

    void insert(int l, int r) {
        dsf.insert(l, r);
        dsr.insert(maxa - r - 1, maxa - l - 1);
    }

    pair<int, int> query() const {
        return min(dsf.query(), dsr.query());
    }

    void rollback() {
        dsf.rollback();
        dsr.rollback();
    }
};

vector<pair<int, int>> upds[maxq << 1];

void update(int o, int l, int r, int ql, int qr, pair<int, int> p) {
    if (ql <= l && r <= qr) {
        upds[o].push_back(p);
        return;
    }
    int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
    if (ql <= mid) {
        update(lc, l, mid, ql, qr, p);
    }
    if (mid < qr) {
        update(rc, mid + 1, r, ql, qr, p);
    }
}

void update(int l, int r, pair<int, int> p) {
    update(1, 0, maxq - 1, l, r, p);
}

void solve(int o, int l, int r, DS2& ds, vector<int>& ans) {
    for (auto& [ql, qr] : upds[o]) {
        ds.insert(ql, qr);
    }
    if (l == r) {
        ans.push_back(ds.query().second);
    } else {
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        solve(lc, l, mid, ds, ans);
        solve(rc, mid + 1, r, ds, ans);
    }
    for (int i = 0; i < sz(upds[o]); i++) {
        ds.rollback();
    }
}

vector<int> solve_queries() {
    DS2 ds;
    vector<int> ans;
    solve(1, 0, maxq - 1, ds, ans);
    return ans;
}

void solve() {
    {
        ST1 st;
        st.update(5, 0);
        dbg(st.query(0, 6));
        DS1 ds;
        ds.insert(1, 5);
        ds.insert(6, 8);
        // ds.insert(2, 4);
        dbg(ds.query().first, ds.query().second);
        // return;
    }
    int q;
    cin >> q;
    multimap<pair<int, int>, int> mp;
    for (int qi = 0; qi < q; qi++) {
        char c;
        int l, r;
        cin >> c >> l >> r;
        if (c == 'A') {
            mp.insert({{l, r}, qi});
        } else {
            auto it = mp.find({l, r});
            assert(it != mp.end());
            update(it->second, qi - 1, {l, r});
            mp.erase(it);
        }
    }
    for (auto& [k, v] : mp) {
        update(v, q - 1, k);
    }
    auto ans = solve_queries();
    ans.resize(q);
    for (auto& a : ans) {
        cout << a << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}