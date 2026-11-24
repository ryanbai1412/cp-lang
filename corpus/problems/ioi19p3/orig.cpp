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

namespace __gnu_pbds {
template <typename T>
using ost =
    tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

namespace s1 {

constexpr int DIRS[4][2] {{0, 1}, {0, -1}, {-1, 0}, {1, 0}};

int n, m;
vector<vector<char>> vis;
vector<vector<int>> arr;

bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y < m;
}

template <typename Cb>
void dfs(int x, int y, const Cb& cb) {
    if (!ibs(x, y) || vis[x][y] || arr[x][y]) {
        return;
    }
    cb(x, y);
    vis[x][y] = true;
    for (auto& [dx, dy] : DIRS) {
        dfs(x + dx, y + dy, cb);
    }
}

long solve(const vector<vector<int>>& _arr) {
    arr = _arr;
    n = sz(arr);
    m = sz(arr[0]);
    vis.resize(n, vector<char>(m));

    long ans = 0;

    auto c2 = [&](long n) -> long { return n * (n - 1) / 2; };

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (vis[i][j] || arr[i][j]) {
                continue;
            }
            int mnx = 1e9, mxx = 0, mny = 1e9, mxy = 0, cnt = 0;
            dfs(i, j, [&](int x, int y) -> void {
                mnx = min(mnx, x);
                mxx = max(mxx, x);
                mny = min(mny, y);
                mxy = max(mxy, y);
                cnt++;
            });

            if (cnt == (mxx - mnx + 1) * (mxy - mny + 1) && mnx && mny &&
                mxx < n - 1 && mxy < m - 1) {
                ans++;
            }
        }
    }

    return ans;
}

}  // namespace s1

struct PSA {
    vector<int> psum;

    PSA(const vector<int>& arr) : psum(sz(arr) + 1) {
        partial_sum(begin(arr), end(arr), psum.begin() + 1);
    }

    int query(int l, int r) const {
        return psum[r] - psum[l];
    }
};

vector<pair<int, int>> solve1(const vector<int>& arr) {
    int n = sz(arr);
    vector<pair<int, int>> ans, st;

    st.emplace_back(arr.back(), n - 1);

    for (int i = n - 2; i >= 0; i--) {
        while (sz(st)) {
            if (i + 1 <= st.back().second - 1) {
                dbg(i + 1, st.back().second - 1);
                ans.emplace_back(i + 1, st.back().second - 1);
            }
            if (st.back().first < arr[i]) {
                st.pop_back();
            } else if (st.back().first == arr[i]) {
                st.pop_back();
                break;
            } else {
                break;
            }
        }
        st.emplace_back(arr[i], i);
    }

    assert(sz(ans) <= 2 * n + 10);
    sort(begin(ans), end(ans));

    return ans;
}

template <typename T>
vector<T> set_inter(const vector<T>& a, const vector<T>& b) {
    vector<T> out;
    set_intersection(begin(a), end(a), begin(b), end(b), back_inserter(out));
    return out;
}

template <typename T>
vector<T> set_diff(const vector<T>& a, const vector<T>& b) {
    vector<T> out;
    set_difference(begin(a), end(a), begin(b), end(b), back_inserter(out));
    return out;
}

struct DS1 {
    int n;
    vector<int> arr;
    vector<ost<pair<int, int>>> v;

    DS1(int n) : n(n), arr(n), v(n + 1) {}

    void reset(int _n) {
        n = _n;
        for (int i = 1; i <= n; i++) {
            v[i].clear();
        }
    }

    template <bool b = true>
    void update(int ind, int x) {
        if (arr[ind] == x) {
            return;
        } else if constexpr (b) {
            if (!x) {
                update<false>(ind, x);
                return;
            }
        }
        int o = ind + 1;
        while (o <= n) {
            v[o].erase({arr[ind], ind});
            if constexpr (b) {
                v[o].insert({x, ind});
            }
            o += o & -o;
        }
        arr[ind] = x;
    }

    int query(int ind, int x) const {
        int ans = 0;
        while (ind > 0) {
            ans += sz(v[ind]) - int(v[ind].order_of_key({x + 1, -1}));
            ind -= ind & -ind;
        }
        return ans;
        // return query(1, 0, n - 1, ind - 1, x);
        // int ans = 0;
        // for (int i = 0; i < ind; i++) {
        //     ans += arr[i] > x;
        // }
        // return ans;
    }
};

constexpr int maxn = 7505;

vector<int> e_arr[maxn];
vector<pair<int, int>> e_queries[maxn];

long solve2(int r_shift,
            int n,
            int m,
            const vector<pair<int, int>>& arr,
            const vector<array<int, 3>>& queries) {
    for (int i = 0; i < m; i++) {
        e_arr[i].clear();
        e_queries[i].clear();
    }
    n -= r_shift;
    for (auto& [ind, x] : arr) {
        e_arr[ind].push_back(x - r_shift);
        dbg(ind, x - r_shift);
    }
    for (auto& [x, ql, qr] : queries) {
        e_queries[ql].emplace_back(x - r_shift, qr);
        dbg(ql, qr, x - r_shift);
    }

    long ans = 0;

    DS1 ds(n);
    vector<int> p_arr;

    for (int i = m - 1; i >= 0; i--) {
        // sort(begin(e_arr[i]), end(e_arr[i]));
        for (auto& x : set_diff(p_arr, e_arr[i])) {
            ds.update(x, 0);
        }
        p_arr = e_arr[i];

        for (auto& a : e_arr[i]) {
            if (!ds.arr[a]) {
                ds.update(a, i);
            }
        }

        for (auto& [x, qr] : e_queries[i]) {
            dbg(x, n);
            ans += ds.query(x + 1, qr - 1);
        }
    }

    return ans;
}

ll count_rectangles(vector<vector<int>> arr) {
    int n = sz(arr), m = sz(arr[0]);

    {
        bool ok = true;
        for (auto& a : arr) {
            for (auto& b : a) {
                ok &= 0 <= b && b <= 1;
            }
        }
        if (ok) {
            return s1::solve(arr);
        }
    }

    vector<pair<int, int>> r_res[n], c_res[m];
    for (int i = 0; i < n; i++) {
        r_res[i] = solve1(arr[i]);
    }
    for (int i = 0; i < m; i++) {
        vector<int> tmp;
        for (int j = 0; j < n; j++) {
            tmp.push_back(arr[j][i]);
        }
        c_res[i] = solve1(tmp);
    }

    vector<int> r_good[m][m];
    for (int i = 0; i < n; i++) {
        for (auto& [ql, qr] : r_res[i]) {
            r_good[ql][qr].push_back(i);
        }
    }

    vector<pair<int, int>> queries[m][m];

    for (int cl = 0; cl < m; cl++) {
        for (int cr = cl; cr < m; cr++) {
            auto& carr = r_good[cl][cr];

            for (int i = 0; i < sz(carr);) {
                int ci = i;
                for (i++; i < sz(carr) && carr[i - 1] + 1 == carr[i]; i++)
                    ;
                int cql = carr[ci], cqr = carr[i - 1];
                cql = max(cql, 1);
                cqr = min(cqr, n - 2);

                queries[cl][cr].emplace_back(cql, cqr);
            }
        }
    }

    long ans = 0;

    int t_bad[n][n];
    vector<pair<int, int>> p_res;

    for (int cl = m - 2; cl >= 1; cl--) {
        vector<pair<int, int>> to_del[m];
        for (auto& [ql, qr] : c_res[cl]) {
            if (!binary_search(begin(p_res), end(p_res),
                               pair<int, int> {ql, qr})) {
                t_bad[ql][qr] = cl + 1;
            }
            to_del[t_bad[ql][qr]].emplace_back(ql, qr);
        }
        p_res = c_res[cl];

        ost<int> ost[n];
        for (auto& [ql, qr] : c_res[cl]) {
            ost[ql].insert(qr);
        }

        for (int cr = cl; cr < m - 1; cr++) {
            for (auto& [ql, qr] : to_del[cr]) {
                ost[ql].erase(qr);
            }
            for (auto& [ql, qr] : queries[cl][cr]) {
                for (int i = ql; i <= qr; i++) {
                    ans += int(ost[i].order_of_key(qr + 1));
                }
            }
        }
    }

    return ans;
}

// The sample grader, reading the input the port reads (corpus/submissions/5661960.cpp is the grader function).
int main() {
    int n, m; scanf("%d %d", &n, &m);
    vector<vector<int>> a(n, vector<int>(m));
    for (auto& row : a) for (auto& x : row) scanf("%d", &x);
    cout << count_rectangles(a) << "\n";
}
