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

struct DS1 {
    vector<pair<int, int>> arr;

    void insert(int l, int r) {
        arr.emplace_back(l, r);
    }

    pair<int, int> query() const {
        pair<int, int> ans {int(1e9), 0};
        for (int i = 0; i < sz(arr); i++) {
            auto& [l2, r2] = arr[i];
            ans = min(ans, {r2 - l2, r2 - l2});
            for (int j = i + 1; j < sz(arr); j++) {
                auto& [l1, r1] = arr[j];
                ans = min(ans, {max(0, r2 - l1), r1 - l2});
            }
        }
        return ans;
    }

    void rollback() {
        arr.pop_back();
    }
};

struct DS2 {
    DS1 dsf, dsr;

    void insert(int l, int r) {
        dsf.insert(l, r);
        dsr.insert(maxa - r, maxa - l);
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