#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
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

constexpr int maxa = 1e7 + 5, maxb = 25;

int c_phi(int x) {
    static int cache[maxa + 1] {};
    if (cache[x]) {
        return cache[x];
    }
    int& ans = cache[x];
    ans = x;
    for (int i = 2; i <= int(1e5); i++) {
        if (x % i != 0) {
            continue;
        }
        ans = ans / i * (i - 1);
        while (x % i == 0) {
            x /= i;
        }
    }
    if (x > 1) {
        ans = ans / x * (x - 1);
    }
    return ans;
}

pair<bool, int> bpow(long base, int exp, int mod) {
    bool of = false, base_of = false;
    long ans = 1;
    while (exp) {
        if (exp & 1) {
            of |= ans * base >= mod || base_of;
            ans = (ans * base) % mod;
        }
        base_of |= base * base >= mod;
        base = (base * base) % mod;
        exp >>= 1;
    }
    return {of, ans};
}

struct SPow {
    static constexpr int logn = 24, low = 18;

    int base, mod, mexp;
    long lift[logn + 1], pcomp[1 << low];

    SPow(int base, int mod) : base(base), mod(mod) {
        {
            long x = 1;
            for (mexp = 0; x < mod; mexp++, x *= base)
                ;
        }
        pcomp[0] = 1;
        for (int i = 1; i < (1 << low); i++) {
            pcomp[i] = (pcomp[i - 1] * base) % mod;
        }
        lift[0] = base;
        for (int i = 1; i <= logn; i++) {
            lift[i] = (lift[i - 1] * lift[i - 1]) % mod;
        }
    }

    pair<bool, int> query(int exp) {
        long ans = pcomp[exp & ((1 << low) - 1)];
        for (int i = logn; i >= low; i--) {
            if ((exp >> i) & 1) {
                ans = (ans * lift[i]) % mod;
            }
        }
        return {exp >= mexp, ans};
    }
};

int base;

pair<bool, int> cpow(int ind, int x, int mod) {
    if (!ind) {
        return {x >= mod, x % mod};
    }
    auto [of, nx] = cpow(ind - 1, x, c_phi(mod));
    if (of) {
        nx += c_phi(mod);
    }
    auto [cf, ans] = bpow(base, nx, mod);
    return {of || cf, ans};
}

int mod;
vector<int> mphis;
vector<SPow> mpows;

struct Node {
    array<int, maxb> v;

    Node operator+(const Node& r) {
        Node ans {};
        for (int i = 0; i < maxb; i++) {
            ans.v[i] = v[i] + r.v[i];
            if (ans.v[i] >= mod) {
                ans.v[i] -= mod;
            }
        }
        return ans;
    }

    Node shift(int ind) const {
        ind = min(ind, maxb);
        Node ans;
        auto ar = copy(v.begin() + ind, v.end(), ans.v.begin());
        fill(ar, ans.v.end(), v.back());
        return ans;
    }

    int operator()(int ind) const {
        return v[min(ind, maxb - 1)];
    }

    static Node c_from(int x) {
        Node nans;
        auto& ans = nans.v;
        vector<int> cmods = mphis;
        vector<pair<bool, int>> cv, nv;
        for (int i = 0; i < maxb; i++) {
            if (!sz(cmods)) {
                ans[i] = ans[i - 1];
                continue;
            }
            if (!i) {
                for (auto& mod : cmods) {
                    cv.emplace_back(x >= mod, x);
                }
                ans[i] = x;
                continue;
            }
            nv.clear();
            cmods.pop_back();
            for (int j = 0; j < sz(cmods); j++) {
                int mod = cmods[j];
                auto [of, nx] = cv[j + 1];
                if (of) {
                    nx += c_phi(mod);
                }
                auto [cf, ans] = mpows[j].query(nx);
                nv.emplace_back(of || cf, ans);
            }
            swap(cv, nv);
            ans[i] = cv[0].second;
        }
        return nans;
    }
};

struct ST {
    int n;
    vector<Node> v, arr;
    vector<int> lazy;

    ST(const vector<Node>& arr) : n(sz(arr)), v(4 * n), arr(arr), lazy(4 * n) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = arr[l];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            v[o] = v[o].shift(x);
            lazy[o] += x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = (v[lc] + v[rc]).shift(lazy[o]);
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    int query(int o, int l, int r, int ql, int qr, int clazy) const {
        if (ql <= l && r <= qr) {
            return v[o](clazy);
        }
        clazy += lazy[o];
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                int ans = query(lc, l, mid, ql, qr, clazy) +
                          query(rc, mid + 1, r, ql, qr, clazy);
                if (ans >= mod) {
                    ans -= mod;
                }
                return ans;
            }
            return query(lc, l, mid, ql, qr, clazy);
        }
        return query(rc, mid + 1, r, ql, qr, clazy);
    }

    int query(int l, int r) const {
        return query(1, 0, n - 1, l, r, 0);
    }
};

void solve() {
    int n, q;
    cin >> n >> q >> mod >> base;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    {
        int x = mod;
        do {
            mphis.push_back(x);
            x = c_phi(x);
        } while (x > 1);
        for (auto& a : mphis) {
            mpows.emplace_back(base, a);
        }
    }
    vector<Node> arr_v;
    for (auto& a : arr) {
        arr_v.push_back(Node::c_from(a));
    }
    //    return;
    ST arr_st(arr_v);
    while (q--) {
        int t, l, r;
        cin >> t >> l >> r;
        l--;
        r--;
        if (t == 0) {
            arr_st.update(l, r, 1);
        } else {
            cout << arr_st.query(l, r) << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}