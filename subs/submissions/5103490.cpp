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

template <int MOD>
struct Mint {
    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            return int(x + MOD);
        }
        return int(x);
    }

    static constexpr Mint from_unchecked(int x) {
        Mint m;
        m.x = x;
        return m;
    }

    int x;

    constexpr Mint(long x = 0) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    Mint& operator-=(const Mint& m) {
        x -= m.x;
        if (x < 0) {
            x += MOD;
        }
        return *this;
    }
    Mint& operator*=(const Mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }

    friend Mint operator+(Mint a, const Mint& b) {
        return a += b;
    }
    friend Mint operator-(Mint a, const Mint& b) {
        return a -= b;
    }
    friend Mint operator*(Mint a, const Mint& b) {
        return a *= b;
    }

    bool operator==(const Mint& m) const {
        return x == m.x;
    }
    bool operator!=(const Mint& m) const {
        return x != m.x;
    }
};

using mint = Mint<int(1e9 + 7)>;

template <size_t N>
using A = array<int, N>;

template <size_t N>
A<N>& operator+=(A<N>& a, const A<N>& b) {
    for (size_t i = 0; i < N; i++) {
        a[i] += b[i];
    }
    return a;
}

template <size_t N>
A<N> operator+(A<N> a, const A<N>& b) {
    return a += b;
}

vector<int> dims;

template <size_t N, typename Cb>
void feach(int ind, A<N> a, const Cb& cb) {
    if (ind == N) {
        cb(a);
        return;
    }
    for (int i = 0; i < dims[ind]; i++) {
        a[ind] = i;
        feach(ind + 1, a, cb);
    }
}

template <size_t N, typename Cb>
void feach(const Cb& cb) {
    feach<N>(0, {}, cb);
}

template <size_t N>
bool ibs(const A<N>& ind) {
    for (size_t i = 0; i < N; i++) {
        if (!(0 <= ind[i] && ind[i] < dims[i])) {
            return false;
        }
    }
    return true;
}

template <size_t N, typename T>
T& index(T* arr, const A<N>& ind) {
    size_t cind = 0;
    for (size_t i = 0; i < N; i++) {
        cind = cind * dims[i] + ind[i];
    }
    return arr[cind];
}

int n;
mint memo[int(1e5 + 5)];

template <size_t D>
mint dp(const A<D>& ind, const A<D>& dd) {
    mint& ans = index(memo, ind);
    if (ans.x != -1) {
        return ans;
    }
    return ans = n + dp(ind + dd, dd);
}

template <size_t D>
void solve() {
    int iarr[n][2];
    for (auto& [i, x] : iarr) {
        cin >> i >> x;
        i--;
    }
    A<D> arr[n] {};
    for (int i = 0; i < n; i++) {
        auto& [qi, x] = iarr[i];
        arr[i][qi] = x;
    }
    A<D> dd {};
    for (auto& a : arr) {
        dd += a;
    }
    memset(memo, -1, sizeof(memo));
    bool missing = false;
    feach<D>([&](const auto& ind) -> void {
        auto cind = ind;
        for (int i = 0; i < n; i++) {
            cind += arr[i];
            if (!ibs(cind)) {
                index<D>(memo, ind) = i + 1;
                return;
            }
        }
        missing = true;
    });
    if (all_of(begin(dd), end(dd),
               [](int x) {
                   return !x;
               }) &&
        missing) {
        cout << -1 << endl;
        return;
    }
    mint ans = 0;
    feach<D>([&](const auto& ind) -> void {
        ans += dp<D>(ind, dd);
    });
    cout << ans << endl;
}

void solve() {
    int kv;
    cin >> n >> kv;
    dims.resize(kv);
    for (auto& a : dims) {
        cin >> a;
    }
    if (kv == 1) {
        solve<1>();
    } else if (kv == 2) {
        solve<2>();
    } else if (kv == 3) {
        solve<3>();
    } else if (kv == 4) {
        solve<4>();
    } else if (kv == 5) {
        solve<5>();
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}