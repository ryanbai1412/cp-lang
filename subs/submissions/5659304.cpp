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

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct DFS1 {
    int n;
    set<int> to_vis;
    vector<char> vis;
    vector<int> arr, ord;

    DFS1(const vector<int>& arr) : n(sz(arr)), vis(n), arr(arr) {
        for (int i = 0; i < n; i++) {
            to_vis.insert(i);
        }
    }

    void dfs(int u) {
        assert(!vis[u]);
        vis[u] = true;
        to_vis.erase(u);

        int ql = arr[u], qr = u;
        if (ql > qr) {
            swap(ql, qr);
        }

        while (true) {
            auto it = to_vis.lower_bound(ql);
            if (it == to_vis.end() || *it > qr) {
                break;
            }
            dfs(*it);
        }
        ord.push_back(u);
    }
};

char a_buf[200 << 20];
size_t a_ind = sizeof(a_buf);

template <typename T>
struct A {
    using value_type = T;

    T* allocate(size_t n) {
        a_ind -= n * sizeof(T);
        a_ind &= -alignof(T);
        return reinterpret_cast<T*>(a_buf + a_ind);
    }

    void deallocate(T*, size_t) {}
};

basic_string<int, char_traits<int>, A<int>> v[1 << 21];
bool v_vis[1 << 21];

struct ST1 {
    int n;

    ST1(int n) : n(1 << 20) {
        v_vis[0] = true;
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            v[o].push_back(x);
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
        r++;
        l += n;
        r += n;
        while (l < r) {
            if (l & 1) {
                v[l++].push_back(x);
            }
            if (r & 1) {
                v[--r].push_back(x);
            }
            l >>= 1;
            r >>= 1;
        }
    }

    template <typename Cb>
    void query(int o, int l, int r, int ind, const Cb& cb) {
        while (sz(v[o])) {
            int cx = v[o].back();
            v[o].pop_back();
            cb(cx);
        }
        if (l == r) {
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            query(lc, l, mid, ind, cb);
        } else {
            query(rc, mid + 1, r, ind, cb);
        }
    }

    template <typename Cb>
    void query(int ind, const Cb& cb) {
        int o = ind + n;
        while (!v_vis[o]) {
            while (sz(v[o])) {
                int cx = v[o].back();
                v[o].pop_back();
                cb(cx);
            }
            o >>= 1;
        }
        while (!v_vis[o]) {
            v_vis[o] = true;
            o >>= 1;
        }
        // query(1, 0, n - 1, ind, cb);
    }
};

struct DFS2 {
    int n;
    vector<char> vis;
    vector<int> ord;
    ST1 st;

    DFS2(const vector<int>& arr) : n(sz(arr)), vis(n), st(n) {
        for (int i = 0; i < n; i++) {
            int ql = arr[i], qr = i;
            if (ql > qr) {
                swap(ql, qr);
            }
            st.update(ql, qr, i);
        }
    }

    void dfs(int u) {
        assert(!vis[u]);
        vis[u] = true;
        st.query(u, [&](int v) -> void {
            if (!vis[v]) {
                dfs(v);
            }
        });
        ord.push_back(u);
    }
};

vector<vector<int>> scc(const vector<int>& arr) {
    int n = sz(arr);

    clock_t c_start = clock();
    DFS1 dfs1(arr);
    for (int i = 0; i < n; i++) {
        if (dfs1.vis[i]) {
            continue;
        }
        dfs1.dfs(i);
    }
    auto ord = dfs1.ord;
    reverse(begin(ord), end(ord));
    dbg(double(clock() - c_start) / CLOCKS_PER_SEC);

    DFS2 dfs2(arr);
    vector<vector<int>> comps;
    for (auto& a : ord) {
        if (dfs2.vis[a]) {
            continue;
        }
        dfs2.ord.clear();
        dfs2.dfs(a);
        comps.push_back(dfs2.ord);
    }

    return comps;
}

int n;
vector<int> cl, cr, l_len, l_v, r_len, r_v;
vector<long> memo;

long dp(int ind) {
    long& ans = memo[ind];
    if (ans != -1) {
        return ans;
    }

    if (l_v[ind] == -1 && r_v[ind] == -1) {
        return l_len[ind] + r_len[ind];
    }

    ans = 1e18;

    if (l_v[ind] != -1) {
        ans = min(ans, l_len[ind] + dp(l_v[ind]));
    }
    if (r_v[ind] != -1) {
        ans = min(ans, r_len[ind] + dp(r_v[ind]));
    }

    return ans;
}

ll minimum_walk(vector<int> arr, int kv) {
    n = sz(arr);

    if (n == 1) {
        return 0;
    }

    int sort_p, sort_s;
    for (sort_p = 0; sort_p < n && arr[sort_p] == sort_p; sort_p++)
        ;
    for (sort_s = n - 1; sort_s >= 0 && arr[sort_s] == sort_s; sort_s--)
        ;

    long ans = 0;
    for (int i = 0; i < n; i++) {
        ans += 2 * max(0, arr[i] - i);
    }

    // lol the testcases, my code TLEs kv != 0
    if (kv == 0) {
        for (int i = 0, j = 0; i < sort_s; i++) {
            if (j < i) {
                j++;
                ans += 2;
            }
            j = max(j, arr[i]);
        }
        return ans;
    }

    auto comps = scc(arr);

    for (auto& a : comps) {
        for (auto& b : a) {
            cerr << b << " ";
        }
        cerr << endl;
    }

    cl = vector<int>(n, n);
    cr = vector<int>(n, -1);

    for (int it = sz(comps) - 1; it >= 0; it--) {
        auto& comp = comps[it];
        int ql = n, ml = n, qr = 0, mr = 0;

        for (auto& a : comp) {
            ml = min(ml, a);
            mr = max(mr, a);
            ql = min(ql, arr[a]);
            qr = max(qr, arr[a]);
        }

        ml = min(ml, ql);
        mr = max(mr, qr);

        for (auto& a : comp) {
            cl[a] = ml;
            cr[a] = mr;
        }
    }

    l_len = vector<int>(n);
    l_v = vector<int>(n);
    r_len = vector<int>(n);
    r_v = vector<int>(n);

    {
        fill(l_v.begin(), l_v.begin() + min(n, sort_p + 1), -1);
        for (int i = sort_p + 1; i < n; i++) {
            int ql = cl[i], qr = cr[i];

            if (ql <= sort_p) {
                l_len[i] = 0;
                l_v[i] = -1;
                continue;
            }

            if (qr < cr[ql - 1]) {
                l_len[i] = 1;
                l_v[i] = ql - 1;
                continue;
            }

            int plen = l_len[ql - 1], pv = l_v[ql - 1];
            if (pv == -1) {
                l_len[i] = plen + 1;
                l_v[i] = -1;
                continue;
            }

            if (qr < cr[pv]) {
                l_len[i] = plen + 1;
                l_v[i] = pv;
                continue;
            }

            l_len[i] = plen + 1 + l_len[pv];
            l_v[i] = l_v[pv];
        }
    }
    {
        fill(r_v.begin() + max(0, sort_s), r_v.end(), -1);
        for (int i = sort_s - 1; i >= 0; i--) {
            int ql = cl[i], qr = cr[i];

            if (sort_s <= qr) {
                r_len[i] = 0;
                r_v[i] = -1;
                continue;
            }

            if (cl[qr + 1] < ql) {
                r_len[i] = 1;
                r_v[i] = qr + 1;
                continue;
            }

            int plen = r_len[qr + 1], pv = r_v[qr + 1];
            if (pv == -1) {
                r_len[i] = plen + 1;
                r_v[i] = -1;
                continue;
            }

            if (cl[pv] < ql) {
                r_len[i] = plen + 1;
                r_v[i] = pv;
                continue;
            }

            r_len[i] = plen + 1 + r_len[pv];
            r_v[i] = r_v[pv];
        }
    }

    for (int i = 0; i < n; i++) {
        // dbg(i, cl[i], cr[i], l_len[i], l_v[i], r_len[i], r_v[i]);
    }

    memo = vector<long>(n, -1);
    // dbg(dp(kv));
    ans += 2 * dp(kv);

    return ans;
}