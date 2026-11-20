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

constexpr int BSIZE = 500;

struct DS {
    struct Block {
        long pl_sum;
        vector<int> arr;
        vector<long> psum;

        Block(long pl_sum, int n) : pl_sum(pl_sum), arr(n), psum(n + 1) {}

        void build() {
            for (int i = 0; i < sz(arr); i++) {
                psum[i + 1] = psum[i] + arr[i];
            }
        }

        void update(int ind, int x) {
            arr[ind] = x;
            build();
        }
    };

    vector<Block> blocks;

    DS(int n) {
        n += 5;

        long pl_sum = 0;

        for (int bl = 0; bl < n; bl += BSIZE) {
            int br = min(n, bl + BSIZE);

            blocks.emplace_back(pl_sum, br - bl);

            pl_sum += blocks.back().psum.back();
        }
    }

    void update(int ind, int x) {
        int bi = ind / BSIZE, bl = bi * BSIZE;

        blocks[bi].update(ind - bl, x);

        long pl_sum = 0;

        for (int bi = 0; bi < sz(blocks); bi++) {
            blocks[bi].pl_sum = pl_sum;
            pl_sum += blocks[bi].psum.back();
        }
    }

    long query_pref(int ind) {
        int bi = ind / BSIZE, bl = bi * BSIZE;

        return blocks[bi].pl_sum + blocks[bi].psum[ind - bl];
    }

    long query(int ql, int qr) {
        return query_pref(qr) - query_pref(ql);
    }
};

struct DS2 {
    struct Block {
        long m_sum;
        vector<int> arr;

        Block(int n) : m_sum(0), arr(n) {}

        void update(int ind, int x) {
            m_sum += arr[ind] - x;
            arr[ind] = x;
        }
    };

    vector<Block> blocks;

    DS2(int n) {
        n += 5;

        for (int bl = 0; bl < n; bl += BSIZE) {
            int br = min(n, bl + BSIZE);

            blocks.emplace_back(br - bl);
        }
    }

    void update(int ind, int x) {
        int bi = ind / BSIZE, bl = bi * BSIZE;

        blocks[bi].update(ind - bl, x);
    }

    long query_pref(int ind) {
        int bi = ind / BSIZE, bl = bi * BSIZE;

        long ans = 0;

        for (int i = 0; i < bi; i++) {
            ans += blocks[i].m_sum;
        }
        for (int i = 0; i < ind - bl; i++) {
            ans += blocks[bi].arr[i];
        }

        return ans;
    }

    long query(int ql, int qr) {
        return query_pref(qr) - query_pref(ql);
    }
};

void solve() {
    int n, q;
    cin >> n >> q;

    vector<array<int, 5>> queries(q);
    for (auto& [ty, ql, qr, d, x] : queries) {
        cin >> ty;
        if (ty == 1 || ty == 2) {
            cin >> ql >> qr >> d >> x;
            ql--;
            qr--;
        } else {
            cin >> ql;
            ql--;
        }
    }

    int max_set[n] {};
    long qans[q] {};

    vector<int> v_queries[n];

    for (int qi = 0; qi < q; qi++) {
        auto& [ty, ql, qr, qd, x] = queries[qi];

        if (ty != 3) {
            continue;
        }

        v_queries[ql].push_back(qi);
    }

    DS ds(n);

    vector<pair<int, int>> evt_add[n], evt_sub[n];

    for (int d = 1; d < BSIZE; d++) {
        for (int i = 0; i < n; i++) {
            evt_add[i].clear();
            evt_sub[i].clear();
        }

        for (int qi = 0; qi < q; qi++) {
            auto& [ty, ql, qr, qd, x] = queries[qi];

            if (ty != 1 || d != qd) {
                continue;
            }

            evt_add[ql].emplace_back(qi, x);
            evt_sub[qr].emplace_back(qi, x);
        }

        for (int r = 0; r < d; r++) {
            for (int i = r; i < n; i += d) {
                for (auto& [qi, x] : evt_add[i]) {
                    dbg(qi, x);
                    ds.update(qi, x);
                }

                for (auto& qi : v_queries[i]) {
                    dbg(i, qi);
                    qans[qi] += ds.query(max_set[i], qi);
                }

                for (auto& [qi, x] : evt_sub[i]) {
                    dbg(qi, -x);
                    ds.update(qi, 0);
                }
            }
        }
    }

    {
        DS2 ds(n);

        vector<pair<int, int>> evts[n];

        for (int qi = 0; qi < q; qi++) {
            auto& [ty, ql, qr, qd, x] = queries[qi];

            if (qd >= BSIZE) {
                for (int i = ql; i <= qr; i += qd) {
                    evts[i].emplace_back(qi, x);
                }
            }
        }

        for (int i = 0; i < n; i++) {
            for (auto& [qi, x] : evts[i]) {
                ds.update(qi, x);
            }

            for (auto& qi : v_queries[i]) {
                qans[qi] += ds.query(max_set[i], qi);
            }

            for (auto& [qi, x] : evts[i]) {
                ds.update(qi, 0);
            }
        }
    }

    for (int qi = 0; qi < q; qi++) {
        if (queries[qi][0] != 3) {
            continue;
        }

        cout << qans[qi] << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}