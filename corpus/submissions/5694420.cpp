#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

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

struct Dungeon {
    int s, p, w, l;
};

struct DS1 {
    struct Lift {
        int mn_esc, lose_d;
        long add;
    };

    static constexpr int LOGN = 25, BSIZE = 5;

    int n, kv;
    vector<Lift> lift[LOGN];

    DS1() {}
    void build(int _kv, const vector<Dungeon>& arr) {
        n = sz(arr);
        kv = _kv;

        for (auto& a : lift) {
            a.resize(n + 1);
        }

        int up_bound = kv == -1 ? 0 : 1 << (kv + 1);
        for (int i = 0; i < n; i++) {
            auto& [s, p, w, l] = arr[i];

            if (__lg(s) < kv || kv == -1) {
                lift[0][i] = {w == n ? 0 : max(0, up_bound - s), w, s};
                continue;
            } else if (__lg(s) > kv) {
                lift[0][i] = {l == n ? 0 : max(0, up_bound - p), l, p};
                continue;
            }

            lift[0][i] = {l == n ? 0 : max(0, min(s, up_bound - p)), l, p};
        }

        lift[0][n] = {0, n, 0};

        for (int it = 1; it < LOGN; it++) {
            for (int i = 0; i <= n; i++) {
                auto& ql = lift[it - 1][i];
                auto& qr = lift[it - 1][ql.lose_d];

                lift[it][i] = {
                    int(max(long(0), min(long(ql.mn_esc), qr.mn_esc - ql.add))),
                    qr.lose_d, ql.add + qr.add};
            }

            if (it % BSIZE == 0) {
                for (int i = it - BSIZE + 1; i < it; i++) {
                    vector<Lift>().swap(lift[i]);
                }
            }
        }
        for (int i = 0; i < LOGN; i++) {
            if (i % BSIZE) {
                vector<Lift>().swap(lift[i]);
            }
        }
    }

    void advance(int& u, long& w, const vector<Dungeon>& arr) {
        if (u == n || (kv != -1 && __lg(w) > kv)) {
            return;
        }
        // assert(__lg(w) == kv);

        for (int it = LOGN - 1; it >= 0; it--) {
            if (!sz(lift[it])) {
                continue;
            }

            for (int it2 = 0; it2 < (1 << BSIZE); it2++) {
                auto& cq = lift[it][u];

                if (kv != -1 && w >= cq.mn_esc) {
                    break;
                }

                u = cq.lose_d;
                w += cq.add;
                dbg(it, u, w);
            }
        }

        if (kv == -1) {
            assert(u == n);
            return;
        }

        dbg(kv, u, w);
        // assert(u != n && __lg(w) == kv);

        dbg(u, arr[u].s, arr[u].p);
        if (w >= arr[u].s) {
            w += arr[u].s;
            u = arr[u].w;
        } else {
            w += arr[u].p;
            u = arr[u].l;
        }

        // auto& cq = lift[0][u];

        // u = cq.win_d;
        // w += cq.win_add;

        dbg(u, w);
        // assert(u == n || __lg(w) > kv);
    }
};

struct Solver {
    static constexpr int LOGN = 25;

    int n;
    vector<Dungeon> arr;
    DS1 lift[LOGN], lift_last;

    Solver() {}
    void build(const vector<Dungeon>& _arr) {
        arr = _arr;
        n = sz(arr);

        for (int i = 0; i < LOGN; i++) {
            lift[i].build(i, arr);
        }
        lift_last.build(-1, arr);
    }

    long query(int u, long kv) {
        for (auto& a : lift) {
            a.advance(u, kv, arr);
        }
        lift_last.advance(u, kv, arr);

        assert(u == n);
        return kv;
    }
} solver;

void init(int n,
          vector<int> arr_s,
          vector<int> arr_p,
          vector<int> arr_w,
          vector<int> arr_l) {
    vector<Dungeon> arr;
    for (int i = 0; i < n; i++) {
        arr.push_back({arr_s[i], arr_p[i], arr_w[i], arr_l[i]});
    }

    solver.build(arr);
    dbg(&solver.arr);

    // size_t c = 0;
    // for (auto& a : solver.lift) {
    //     for (auto& b : a.lift) {
    //         if (sz(b) != b.capacity()) {
    //             cout << sz(b) << " " << b.capacity() << endl;
    //         }
    //         c += b.capacity() * sizeof(DS1::Lift);
    //     }
    // }
    // cout << (c >> 20) << endl;
    // exit(0);
}

ll simulate(int u, int kv) {
    dbg(&solver.arr);
    return solver.query(u, kv);
}