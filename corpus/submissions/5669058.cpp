#include <algorithm>
#include <cstdio>
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

struct FastCin {
    char buf[40 << 20], *i = buf;

    FastCin() {
        std::ignore = fread_unlocked(buf, sizeof(buf), 1, stdin);
    }

    FastCin& operator>>(int& x) {
        while (*i < '0') {
            i++;
        }
        x = 0;
        while (*i >= '0') {
            x = x * 10 + (*(i++) ^ '0');
        }
        return *this;
    }
} in;

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

struct PQ {
    vector<int> del_cnt;
    priority_queue<pair<int, int>> pq;

    PQ(int n) : del_cnt(n) {}

    pair<int, int> top() {
        while (true) {
            auto p = pq.top();
            if (del_cnt[p.second]) {
                del_cnt[p.second]--;
                pq.pop();
            } else {
                return p;
            }
        }
    }

    void erase(int ind) {
        del_cnt[ind]++;
    }

    void emplace(int a, int b) {
        pq.emplace(a, b);
    }
};

struct DS1 {
    struct Maint {
        int n, me_col, c_size = 0;
        const vector<array<int, 3>>& arr;
        PQ diff[3];

        Maint(int me_col, const vector<array<int, 3>>& arr)
            : n(sz(arr)), me_col(me_col), arr(arr), diff {n, n, n} {}

        PQ& get_me() {
            assert(me_col != -1);
            return diff[me_col];
        }

        int get_val(int ind, int col) {
            if (col == me_col) {
                return -arr[ind][col];
            } else if (me_col == -1) {
                return arr[ind][col];
            }
            return arr[ind][col] - arr[ind][me_col];
        }

        void insert(int ind) {
            c_size++;
            for (int col = 0; col < 3; col++) {
                diff[col].emplace(get_val(ind, col), ind);
            }
        }

        void erase(int ind) {
            c_size--;
            for (int col = 0; col < 3; col++) {
                diff[col].erase(ind);
            }
        }

        int size() const {
            return c_size;
        }
    };

    long c_flow = 0;
    array<int, 3> cap;
    vector<array<int, 3>> arr;
    Maint used[3], unused;

    DS1(int cap2, const vector<array<int, 3>>& arr)
        : cap({0, 0, cap2}),
          arr(arr),
          used {Maint(0, arr), Maint(1, arr), Maint(2, arr)},
          unused(-1, arr) {
        int n = sz(arr), ord[n];
        iota(ord, ord + n, 0);
        nth_element(ord, ord + cap2, ord + n,
                    CmpByKey([&](int u) -> int { return -arr[u][2]; }));

        bool iused[n] {};
        for (int i = 0; i < cap2; i++) {
            iused[ord[i]] = true;
        }

        for (int i = 0; i < n; i++) {
            if (iused[i]) {
                cap[2]--;
                c_flow += arr[i][2];
                used[2].insert(i);
            } else {
                unused.insert(i);
            }
        }
        // for (int i = 0; i < sz(arr); i++) {
        //     unused.insert(i);
        // }
        // for (int i = 0; i < cap2; i++) {
        //     move_best(2);
        // }
    }

    Maint& get_maint(int c) {
        if (c == -1) {
            return unused;
        }
        return used[c];
    }

    int get_val(int ind, int col) {
        if (col == -1) {
            return 0;
        }
        return arr[ind][col];
    }

    void m_move(int ind, int c1, int c2) {
        dbg("MOVE", ind, c1, c2);
        if (c1 != -1) {
            cap[c1]++;
        }
        if (c2 != -1) {
            cap[c2]--;
        }

        c_flow += get_val(ind, c2) - get_val(ind, c1);
        get_maint(c1).erase(ind);
        get_maint(c2).insert(ind);
    }

    void erase_worst(int col) {
        m_move(used[col].get_me().top().second, col, -1);
    }

    void move_best(int col) {
        m_move(unused.diff[col].top().second, -1, col);
    }

    void flow() {
        pair<long, basic_string<int>> opt {-1, {}};

        int perm[3];
        iota(begin(perm), end(perm), 0);

        do {
            int prv = -1;
            long c_ans = 0;
            bool bad = false;
            basic_string<int> perm_vis;

            for (auto& a : perm) {
                auto [w, qi] = get_maint(prv).diff[a].top();

                c_ans += w;
                perm_vis.push_back(a);

                if (cap[a]) {
                    break;
                }
                if (!sz(get_maint(a))) {
                    bad = true;
                    break;
                }

                prv = a;
            }

            if (c_flow == 47 && perm[0] == 0 && perm[1] == 2 && perm[2] == 1) {
                dbg(bad, c_ans, prv);
            }
            if (bad || (sz(perm_vis) == 3 && !cap[perm[2]])) {
                continue;
            }

            opt = max(opt, {c_ans, perm_vis});
        } while (next_permutation(begin(perm), end(perm)));

        auto [w, ord] = opt;
        dbg(w);
        assert(w != -1);

        int prv = -1;
        for (auto& a : ord) {
            m_move(get_maint(prv).diff[a].top().second, prv, a);
            prv = a;
        }
    }
};

void solve() {
    int n, kv1, kv2;
    in >> n >> kv1 >> kv2;
    int arr[n][2];
    for (int j = 0; j < 2; j++) {
        for (int i = 0; i < n; i++) {
            in >> arr[i][j];
        }
    }

    vector<array<int, 3>> v_arr;
    for (auto& [x, y] : arr) {
        v_arr.push_back({x, y, x + y});
    }

    DS1 ds(kv1, v_arr);

    // vector<int> i_ord(n);
    // iota(begin(i_ord), end(i_ord), 0);
    // nth_element(begin(i_ord), i_ord.begin() + kv1, end(i_ord),
    //             CmpByKey([&](int u) -> int { return -v_arr[u][2]; }));
    // for (auto& a : i_ord) {
    //     ds.m_move(a, -1, 2);
    // }
    // for (int i = 0; i < kv1; i++) {
    //     ds.move_best(2);
    // }
    // cout << ds.c_flow << endl;
    // return;

    assert((ds.cap == array<int, 3> {0, 0, 0}));
    assert(sz(ds.used[2]) == kv1);

    long ans = ds.c_flow;
    dbg(ans);

    while (sz(ds.used[2]) > kv2 && sz(ds.unused)) {
        ds.erase_worst(2);
        dbg(ds.c_flow);

        ds.cap[2]--;
        ds.cap[0]++;
        ds.cap[1]++;
        dbg(ds.cap[0], ds.cap[1], ds.cap[2]);

        ds.flow();
        dbg(ds.c_flow);
        ds.flow();

        ans = max(ans, ds.c_flow);
        dbg(ds.c_flow);
    }

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    in >> tcs;
    while (tcs--) {
        solve();
    }
}