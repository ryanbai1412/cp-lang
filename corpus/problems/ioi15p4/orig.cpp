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

using i128 = __int128_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T, typename Op = plus<T>>
struct ST {
    int n;
    vector<T> v, arr;
    Op op;

    ST() {}
    ST(const vector<T>& arr, const Op& op = {})
        : n(sz(arr)), v(4 * n), arr(arr), op(op) {
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
        v[o] = op(v[lc], v[rc]);
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
        v[o] = op(v[lc], v[rc]);
    }

    void update(int ind, const T& x) {
        update(1, 0, n - 1, ind, x);
    }

    T query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }

        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return op(query(lc, l, mid, ql, qr),
                          query(rc, mid + 1, r, ql, qr));
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    T query(int l, int r) const {
        dbg(l, r);
        return query(1, 0, n - 1, l, r);
    }
};

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

    static mint from_unchecked(int x) {
        mint m;
        m.x = x;
        return m;
    }

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    constexpr mint operator-() const {
        if (!x) {
            return {};
        }
        return mint::from_unchecked(MOD - x);
    }

    constexpr mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    constexpr mint& operator-=(const mint& m) {
        return *this += -m;
    }

    constexpr mint& operator*=(const mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }

    friend constexpr mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend constexpr mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend constexpr mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

template <typename T>
struct MaxOp {
    T operator()(const T& a, const T& b) const {
        return max(a, b);
    }
};

struct Solver {
    int n;
    vector<int> arr_x, arr_y;
    set<int> pos_x;
    ST<mint, multiplies<mint>> st_x;
    ST<int, MaxOp<int>> st_y;

    Solver() {}
    Solver(const vector<int>& arr_x, const vector<int>& arr_y)
        : n(sz(arr_x)),
          arr_x(arr_x),
          arr_y(arr_y),
          st_x(({
              vector<mint> v(n);
              for (int i = 0; i < n; i++) {
                  v[i] = arr_x[i];
              }
              v;
          })),
          st_y(arr_y) {
        for (int i = 0; i < n; i++) {
            if (arr_x[i] > 1) {
                pos_x.insert(i);
            }
        }
    }

    int query() {
        long opt = 0;
        mint opt_m = 0;

        auto upd = [&](long co, mint cm) -> void {
            if (co > opt) {
                opt = co;
                opt_m = cm;
            }
        };

        int prv = n;
        auto go = [&](int ind) -> void {
            mint pref_x = st_x.query(0, ind);
            int max_y = st_y.query(ind, prv - 1);
            upd(max_y, pref_x * max_y);
            dbg(ind, max_y);

            prv = ind;
            opt *= arr_x[ind];
        };

        for (auto it = pos_x.rbegin(); it != pos_x.rend() && opt < long(1e9);
             ++it) {
            go(*it);
        }

        if (prv && opt < long(1e9)) {
            assert((!sz(pos_x) && prv == n) ||
                   (sz(pos_x) && prv == *pos_x.begin()));

            mint pref_x = 1;
            int max_y = st_y.query(0, prv - 1);
            upd(max_y, pref_x * max_y);
        }

        return opt_m.x;
    }

    void update_x(int ind, int x) {
        if (arr_x[ind] > 1) {
            pos_x.erase(ind);
        }

        arr_x[ind] = x;
        st_x.update(ind, x);

        if (arr_x[ind] > 1) {
            pos_x.insert(ind);
        }
    }

    void update_y(int ind, int x) {
        arr_y[ind] = x;
        st_y.update(ind, x);
    }
} solver;

int init(int n, int arr_x[], int arr_y[]) {
    solver =
        Solver(vector<int>(arr_x, arr_x + n), vector<int>(arr_y, arr_y + n));
    return solver.query();
}

int updateX(int ind, int x) {
    solver.update_x(ind, x);
    return solver.query();
}

int updateY(int ind, int x) {
    solver.update_y(ind, x);
    return solver.query();
}
// The original is the functions the IOI grader calls. This `main` stands in for
// the grader: n, x, y, m, then m updates `kind year value` (1 sets x, 2 sets y).
int main() {
	int n;
	if (scanf("%d", &n) != 1) return 1;
	vector<int> x(n), y(n);
	for (auto& v : x) scanf("%d", &v);
	for (auto& v : y) scanf("%d", &v);
	printf("%d\n", init(n, x.data(), y.data()));
	int m;
	scanf("%d", &m);
	while (m--) {
		int kind, at, value;
		scanf("%d %d %d", &kind, &at, &value);
		printf("%d\n", kind == 1 ? updateX(at, value) : updateY(at, value));
	}
}
