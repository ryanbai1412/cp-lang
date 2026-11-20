#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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
#define sz(x) int(std::size(x))

struct NV {
    long matched, free_l, free_r;

    NV operator+(const NV& n) const {
        long sub = min(free_l, n.free_r);
        return {matched + n.matched + sub, free_l + n.free_l - sub,
                free_r + n.free_r - sub};
    }
};

struct ST {
    int n;
    vector<NV> v;

    ST(int n) : n(n), v(4 * n) {}

    void update(int o, int l, int r, int ind, const NV& x) {
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

    void update(int ind, const NV& x) {
        update(1, 0, n - 1, ind, x);
    }

    NV query_all() const {
        return v[1];
    }
};

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n + 1);
    vector<pair<long, long>> s_arr(n + 1);
    for (int i = 1; i <= n; i++) {
        cin >> arr[i];
    }
    ST st(n + 1);
    long twos = 0;
    auto upd_st = [&](int ind) -> void {
        auto& [x, f] = s_arr[ind];
        st.update(ind, NV {0, x % 2, 0} + NV {0, 0, f});
        NV cnv = NV {0, x % 2, 0} + NV {0, 0, f};
        dbg(ind, x, f, cnv.free_l, cnv.free_r);
    };
    auto upd = [&](int ind, int mul) -> void {
        s_arr[ind].first += mul * arr[ind];
        twos += mul * (arr[ind] / 2);
        s_arr[min(n, 2 * ind - 1)].second += mul * (arr[ind] / 2);
        upd_st(ind);
        upd_st(min(n, 2 * ind - 1));
    };
    for (int i = 1; i <= n; i++) {
        upd(i, +1);
    }
    while (q--) {
        int ind, x;
        cin >> ind >> x;
        upd(ind, -1);
        arr[ind] += x;
        upd(ind, +1);
        long matched = st.query_all().matched;
        dbg(matched);
        cout << matched + (twos - matched) * 2 / 3 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}