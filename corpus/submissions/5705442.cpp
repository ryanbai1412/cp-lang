#pragma GCC optimize("Ofast")
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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
void sort_unique(vector<T>& arr) {
    sort(begin(arr), end(arr));
    arr.erase(unique(begin(arr), end(arr)), arr.end());
}

constexpr int BSIZE = 100;

struct Bit {
    int n;
    vector<int> arr;
    vector<long> v;

    Bit(int n) : n(n), arr(n), v(n + 1) {}

    void update(int ind, int x) {
        x -= arr[ind];
        arr[ind] += x;

        ind++;
        while (ind <= n) {
            v[ind] += x;
            ind += ind & -ind;
        }
    }

    long query_pref(int ind) const {
        long ans = 0;

        while (ind > 0) {
            ans += v[ind];
            ind -= ind & -ind;
        }

        return ans;
    }

    long query(int ql, int qr) const {
        return query_pref(qr) - query_pref(ql);
    }
};

struct DS {
    struct Block {
        int n;
        long pl_sum;
        int* arr;
        long* psum;

        Block(long pl_sum, int n, int* arr, long* psum)
            : n(n), pl_sum(pl_sum), arr(arr), psum(psum) {}

        void build() {
            psum[0] = 0;
            for (int i = 0; i < n; i++) {
                psum[i + 1] = psum[i] + arr[i];
            }
        }

        void update(int ind, int x) {
            arr[ind] = x;
            build();
        }
    };

    int n, BSIZE, lg_bsize;
    vector<Block> blocks;
    vector<int> b_arr;
    vector<long> b_psum;

    DS(int n) : BSIZE(max(10, int(sqrt(n)))) {
        BSIZE = 1 << (__lg(BSIZE) + 1);
        lg_bsize = __lg(BSIZE);
        n += 5;
        this->n = n;

        long pl_sum = 0;

        {
            int cnt_blocks = n / BSIZE + 1;
            b_arr.resize(cnt_blocks * BSIZE);
            b_psum.resize(cnt_blocks * (BSIZE + 1));
        }

        for (int bi = 0, bl = 0; bl < n; bl += BSIZE, bi++) {
            int br = min(n, bl + BSIZE);

            blocks.emplace_back(pl_sum, br - bl, b_arr.data() + BSIZE * bi,
                                b_psum.data() + (BSIZE + 1) * bi);

            pl_sum += blocks.back().psum[br - bl];
        }
    }

    void update(int ind, int x) {
        int bi = ind / BSIZE, bl = bi * BSIZE;

        blocks[bi].update(ind - bl, x);

        long pl_sum = 0;

        for (int bi = 0; bi < sz(blocks); bi++) {
            int bl = bi * BSIZE, br = min(n, bl + BSIZE);

            blocks[bi].pl_sum = pl_sum;
            pl_sum += blocks[bi].psum[br - bl];
        }
    }

    long query_pref(int ind) {
        int bi = ind >> lg_bsize, bl = bi << lg_bsize;

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
            m_sum += x - arr[ind];
            arr[ind] = x;
        }
    };

    vector<Block> blocks;

    DS2(int n) {
        n += BSIZE + 100;

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
    {
        DS2 ds(10);
        ds.update(1, 5);
        ds.update(2, 10);
        ds.update(5, 6);
        for (int i = 0; i < 10; i++) {
            dbg(ds.query_pref(i));
        }
    }

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

    vector<pair<int, int>> evt_add[n], evt_sub[n];
    int cnt1 = 0, cnt2 = 0;

    for (int d = 1; d < BSIZE; d++) {
        for (int i = 0; i < n; i++) {
            evt_add[i].clear();
            evt_sub[i].clear();
        }

        vector<int> pts;

        for (int qi = 0; qi < q; qi++) {
            auto& [ty, ql, qr, qd, x] = queries[qi];

            if (ty != 1 || d != qd) {
                continue;
            }

            pts.push_back(qi);
            evt_add[ql].emplace_back(qi, x);
            evt_sub[qr].emplace_back(qi, x);
        }

        sort_unique(pts);

        int comp[q];
        for (int i = 0, j = 0; i < q; i++) {
            while (j < sz(pts) && pts[j] < i) {
                j++;
            }
            comp[i] = j;
        }

        DS ds(sz(pts));

        int m_ds = 0;

        for (int r = 0; r < d; r++) {
            for (int i = r; i < n; i += d) {
                for (auto& [qi, x] : evt_add[i]) {
                    m_ds++;
                    cnt1++;
                    ds.update(comp[qi], x);
                }

                if (!m_ds) {
                    continue;
                }
                for (auto& qi : v_queries[i]) {
                    cnt2++;
                    qans[qi] += ds.query(comp[max_set[i]], comp[qi]);
                }

                for (auto& [qi, x] : evt_sub[i]) {
                    m_ds--;
                    ds.update(comp[qi], 0);
                }
            }
        }
    }

    dbg(cnt1, cnt2);

    {
        DS2 ds(q);

        vector<pair<int, int>> evts[n];

        for (int qi = 0; qi < q; qi++) {
            auto& [ty, ql, qr, qd, x] = queries[qi];

            if (ty == 1 && qd >= BSIZE) {
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