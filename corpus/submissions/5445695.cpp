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

struct mint {
    static constexpr int MOD = 1e9 + 7;

    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}
    constexpr mint(int x, bool) : x(x) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    mint operator-() const {
        if (!x) {
            return 0;
        }
        return mint(MOD - x, true);
    }

    mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    mint& operator-=(const mint& m) {
        return *this += -m;
    }

    mint& operator*=(const mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }

    friend mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

struct N {
    mint m, b;

    friend N operator+(const N& a, const N& b) {
        return {a.m * b.m, a.m * b.b + a.b};
    }
};

template <typename T>
struct ST {
    int n;
    vector<T> v, arr;

    ST(const vector<T>& arr) : n(sz(arr)), v(4 * n), arr(arr) {
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

    T query_all() const {
        return v[1];
    }
};

mint solve(const vector<int>& arr) {
    int n = sz(arr);
    mint pow3[n];
    pow3[0] = 1;
    for (int i = 1; i < n; i++) {
        pow3[i] = pow3[i - 1] * 3;
    }
    //    mint ans = 0;
    //    for (int i = n - 1; i >= 0; i--) {
    //        mint cmul;
    //        if (i == n - 1) {
    //            cmul = 1;
    //        } else {
    //            cmul = 2 * pow3[n - 2 - i];
    //        }
    //        ans = arr[i] * (cmul + ans);
    //    }
    //    return ans;
    N ans {1, 0};
    for (int i = n - 1; i >= 0; i--) {
        mint cmul;
        if (i == n - 1) {
            cmul = 1;
        } else {
            cmul = 2 * pow3[n - 2 - i];
        }
        ans = N {arr[i], arr[i] * cmul} + ans;
    }
    return ans.b;
}

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    mint pow3[n];
    pow3[0] = 1;
    for (int i = 1; i < n; i++) {
        pow3[i] = pow3[i - 1] * 3;
    }
    auto get_n = [&](int i) -> N {
        mint cmul;
        if (i == n - 1) {
            cmul = 1;
        } else {
            cmul = 2 * pow3[n - 2 - i];
        }
        return {arr[i], arr[i] * cmul};
    };
    vector<N> starr;
    for (int i = 0; i < n; i++) {
        starr.push_back(get_n(i));
    }
    ST st(starr);
    while (q--) {
        int ind, x;
        cin >> ind >> x;
        ind--;
        arr[ind] = x;
        st.update(ind, get_n(ind));
        cout << st.query_all().b << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}