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

template <typename Cb>
struct KeyCmp {
    Cb cb;

    KeyCmp(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

void coord_comp(vector<int*> arr) {
    vector<int> vals;
    for (auto& a : arr) {
        vals.push_back(*a);
    }
    sort(begin(vals), end(vals));
    vals.erase(unique(begin(vals), end(vals)), vals.end());
    for (auto& a : arr) {
        *a = int(lower_bound(begin(vals), end(vals), *a) - vals.begin());
    }
}

struct ST1 {
    int n;
    vector<int> lazy, v;

    ST1(int n) : n(n), lazy(4 * n, -1), v(4 * n, -1) {}

    void update(int o, int l, int r, int ql, int qr, int x) {
        v[o] = max(v[o], x);
        if (ql <= l && r <= qr) {
            lazy[o] = max(lazy[o], x);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    int query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int ans = lazy[o], mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            ans = max(ans, query(lc, l, mid, ql, qr));
        }
        if (mid < qr) {
            ans = max(ans, query(rc, mid + 1, r, ql, qr));
        }
        return ans;
    }

    int query(int l, int r) const {
        return query(1, 0, n - 1, l, r);
    }
};

template <typename T>
struct ST2 {
    int n;
    vector<T> v;

    ST2(int n) : n(n), v(4 * n) {}

    void update(int o, int l, int r, int ind, const T& x) {
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
        v[o] = v[lc] + v[rc];
    }

    void update(int ind, const T& x) {
        update(1, 0, n - 1, ind, x);
    }

    T query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr) +
                       query(rc, mid + 1, r, ql, qr);
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    T query(int l, int r) const {
        assert(0 <= l && l <= r && r < n);
        return query(1, 0, n - 1, l, r);
    }
};

struct NMax {
    int v;

    NMax operator+(const NMax& n) const {
        return {max(v, n.v)};
    }
};

struct PS {
    int yl, yr, x;
};

bool inter(int l1, int r1, int l2, int r2) {
    if (l1 > l2) {
        swap(l1, l2);
        swap(r1, r2);
    }
    return r1 >= l2;
}

int solve1(vector<PS> arr) {
    coord_comp(({
        vector<int*> v;
        for (auto& [yl, yr, _x] : arr) {
            v.push_back(&yl);
            v.push_back(&yr);
        }
        v;
    }));
    int n = sz(arr), ans = 0;
    ST1 st(2 * n);
    for (int i = n - 1; i >= 0; i--) {
        ans = max(ans, st.query(arr[i].yl, arr[i].yr) - arr[i].x);
        st.update(arr[i].yl, arr[i].yr, arr[i].x);
    }
    return ans;
}

vector<int> solve_queries(vector<array<int, 3>> arr,
                          vector<pair<int, int>> queries) {
    int n = sz(arr), q = sz(queries);
    coord_comp(({
        vector<int*> v;
        for (auto& a : arr) {
            v.push_back(&a[1]);
        }
        for (auto& a : queries) {
            v.push_back(&a.second);
        }
        v;
    }));
    vector<tuple<int, bool, int>> evts;
    for (int i = 0; i < n; i++) {
        evts.emplace_back(arr[i][0], false, i);
    }
    for (int i = 0; i < q; i++) {
        evts.emplace_back(queries[i].first, true, i);
    }
    sort(begin(evts), end(evts), greater<>());
    ST2<NMax> st(n + q);
    vector<int> ans(q);
    int opt[n + q] {};
    for (auto& [_x, ty, ind] : evts) {
        if (!ty) {
            opt[arr[ind][1]] = max(opt[arr[ind][1]], arr[ind][2]);
            st.update(arr[ind][1], {opt[arr[ind][1]]});
            // st.update(arr[ind][1], {arr[ind][2]});
        } else {
            ans[ind] = st.query(queries[ind].second, n + q - 1).v;
        }
    }
    // vector<int> ans;
    // for (auto& [x, y] : queries) {
    //     int cans = -1;
    //     for (auto& [x2, y2, v] : arr) {
    //         if (x2 >= x && y2 >= y) {
    //             cans = max(cans, v);
    //         }
    //     }
    //     ans.push_back(cans);
    // }
    return ans;
}

int solve_slow(const vector<PS>& arr) {
    int n = sz(arr), ans = 0;
    for (int i = n - 1; i >= 0; i--) {
        for (int j = i + 1; j < n; j++) {
            if (inter(arr[i].yl, arr[i].yr, arr[j].yl, arr[j].yr) ||
                (arr[j].yl >= arr[i].yr &&
                 arr[j].x - arr[j].yl >= arr[i].x - arr[i].yr)) {
                ans = max(ans, arr[j].x - arr[i].x);
            }
        }
    }
    return ans;
}

int solve(const vector<PS>& arr) {
    if (sz(arr) <= 20) {
        return solve_slow(arr);
    }
    int n = sz(arr), ans = 0;
    vector<array<int, 3>> qarr;
    vector<pair<int, int>> queries;
    for (int i = 0; i < n; i++) {
        qarr.push_back({arr[i].yl, arr[i].x - arr[i].yl, arr[i].x});
        queries.emplace_back(arr[i].yr, arr[i].x - arr[i].yr);
    }
    auto sq = solve_queries(qarr, queries);
    for (int i = 0; i < n; i++) {
        ans = max(ans, sq[i] - arr[i].x);
    }
    return ans;
}

int solve_both(vector<PS> vps) {
    int ans = max(solve1(vps), solve(vps));
    for (auto& [yl, yr, _x] : vps) {
        yl *= -1;
        yr *= -1;
        swap(yl, yr);
    }
    return max(ans, solve(vps));
}

struct NP {
    int yl, yr, y, x;

    NP operator+(const NP& n) const {
        return {min(yl, y + n.yl), max(yr, y + n.yr), y + n.y, x + n.x};
    }
};

int sequence(int n, vector<int> arr) {
    // {
    //     ST2<NP> st(3);
    //     st.update(1, {1, 1, 1, 0});
    //     st.update(2, {1, 1, 1, 0});
    //     st.update(1, {0, 0, 0, 1});
    //     dbg(st.query(1, 2).yr);
    //     return -1;
    // }
    for (auto& a : arr) {
        a--;
    }
    vector<int> inds[n];
    for (int i = 0; i < n; i++) {
        inds[arr[i]].push_back(i);
    }
    ST2<NP> st(n + 1);
    for (int i = 1; i <= n; i++) {
        st.update(i, NP {1, 1, 1, 0});
    }
    int ans = 0;
    for (int xv = 0; xv < n; xv++) {
        if (!sz(inds[xv])) {
            continue;
        }
        for (auto& a : inds[xv]) {
            st.update(a + 1, {0, 0, 0, 1});
        }
        if (sz(inds[xv]) > ans) {
            vector<PS> vps;
            NP cpref {};
            auto go = [&](int ql, int qr) -> void {
                // auto cpref = ql ? st.query(0, ql - 1) : NP {};
                auto cn = st.query(ql, qr - 1);
                vps.push_back(
                    PS {cn.yl + cpref.y, cn.yr + cpref.y, cpref.x + cn.x});
                cpref = cpref + cn;
                dbg(cpref.y, cn.yl, cn.yr);
            };
            go(0, inds[xv][0] + 1);
            for (int i = 0; i < sz(inds[xv]) - 1; i++) {
                go(inds[xv][i] + 1, inds[xv][i + 1] + 1);
            }
            go(inds[xv].back() + 1, n + 1);
            ans = max(ans, solve_both(vps));
        }
        for (auto& a : inds[xv]) {
            st.update(a + 1, {-1, -1, -1, 0});
        }
    }
    return ans;
}