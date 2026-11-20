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

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(const Cb& cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

template <typename T>
void append(vector<T>& a, const vector<T>& b) {
    a.insert(a.end(), begin(b), end(b));
}

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

using P = pair<int, int>;

struct R {
    mt19937_64 cowng =
        mt19937_64(chrono::steady_clock::now().time_since_epoch().count());

    long rint(long l, long r) {
        return cowng() % (r - l + 1) + l;
    }
} R;

struct E {
    int n, ans;
    vector<P> arr;

    E(const vector<P>& arr) : n(sz(arr)), ans(n + 1), arr(arr) {
        for (int i = 0; i < n; i++) {
            ans += get_cost(i);
        }
    }

    int get_cost(int i) const {
        if (!(0 <= i && i < n - 1)) {
            return 0;
        }
        if (i & 1) {
            return (arr[i - 1].second < arr[i].second) !=
                   (arr[i].second < arr[i + 1].second);
        } else {
            return ((i ? arr[i - 1].first : 0) < arr[i].first) !=
                   (arr[i].first < arr[i + 1].first);
        }
    }

    void m_swap(int u, int v) {
        if (u > v) {
            swap(u, v);
        }
        if (u == v) {
            return;
        }
        if (u + 1 == v) {
            for (int i = u - 1; i <= v + 1; i++) {
                ans -= get_cost(i);
            }
            swap(arr[u], arr[v]);
            for (int i = u - 1; i <= v + 1; i++) {
                ans += get_cost(i);
            }
            return;
        }

        for (auto& i : {u, v}) {
            for (auto& di : {-1, 0, 1}) {
                ans -= get_cost(i + di);
            }
        }

        swap(arr[u], arr[v]);

        for (auto& i : {u, v}) {
            for (auto& di : {-1, 0, 1}) {
                ans += get_cost(i + di);
            }
        }
    }

    int simu_swap(int u, int v) {
        m_swap(u, v);
        int r_ans = ans;
        m_swap(u, v);
        return r_ans;
    }
};

int eval(vector<P> arr) {
    int ans = sz(arr) + 1;

    auto get_cost = [&](int i) -> int {
        if (i & 1) {
            return (arr[i - 1].second < arr[i].second) !=
                   (arr[i].second < arr[i + 1].second);
        } else {
            return ((i ? arr[i - 1].first : 0) < arr[i].first) !=
                   (arr[i].first < arr[i + 1].first);
        }
    };

    for (int i = 0; i + 1 < sz(arr); i++) {
        ans += get_cost(i);
    }
    return ans;
}

vector<P> gen(const vector<P>& arr) {
    P p1 = {0, arr[0].second}, p2 = arr[0];
    vector<P> ans {p1};

    for (int i = 1; i < sz(arr); i++) {
        auto& [x, y] = arr[i];
        if (i & 1) {
            if ((p1.first < p2.first) != (p2.first < x)) {
                ans.push_back(p2);
            }
            p1 = {x, p2.second};
            p2 = arr[i];
            ans.push_back(p1);
        } else {
            if ((p1.second < p2.second) != (p2.second < y)) {
                ans.push_back(p2);
            }
            p1 = {p2.first, y};
            p2 = arr[i];
            ans.push_back(p1);
        }
    }

    ans.push_back(p2);

    return ans;
}

struct Ans {
    int len;
    const vector<P>& arr;

    Ans(const E& e) : len(e.ans), arr(e.arr) {}
    Ans(const vector<P>& arr) : len(eval(arr)), arr(arr) {}
};

void print(ostream& out, const vector<P>& ans) {
    out << sz(ans) << endl;
    for (auto& [x, y] : ans) {
        out << x << " " << y << endl;
    }
}

int opt_ans = 1e9;
vector<P> opt_ans_v;

bool print_if_opt(const Ans& ans) {
    if (ans.len >= opt_ans) {
        return false;
    }
    opt_ans = ans.len;
    ofstream out("output.txt");
    dbg(ans.len);
    opt_ans_v = gen(ans.arr);
    // print(out, opt_ans_v);
    return true;
}

void solve() {
    clock_t c_start = clock();

    int n;
    cin >> n;
    vector<P> arr(n);
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    sort(begin(arr), end(arr));
    // tc 2
    if (n == 600) {
        auto run = [&](const vector<P>& arr) -> void {
            E eval(arr);
            for (int iter = 0; iter < 2e6; iter++) {
                int u = R.rint(0, n - 1), v = R.rint(u, min(n - 1, u + 50));
                if (eval.simu_swap(u, v) <= eval.ans) {
                    eval.m_swap(u, v);
                }
            }
            print_if_opt(eval);
        };

        vector<P> ty[2][2];
        for (auto& [x, y] : arr) {
            ty[x < 5000][y < 5000].emplace_back(x, y);
        }

        vector<P> cords[4];
        for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                cords[i * 2 + j] = ty[i][j];
            }
        }

        // for (int mask = 0; mask < (1 << 4); mask++) {
        for (int mask : {0}) {
            int perm[4];
            iota(begin(perm), end(perm), 0);
            do {
                vector<P> carr;
                for (int i = 0; i < 4; i++) {
                    vector<P> cval = cords[perm[i]];
                    if (on(mask, i)) {
                        reverse(begin(cval), end(cval));
                    }
                    append(carr, cval);
                }
                dbg(mask);
                run(carr);
            } while (next_permutation(begin(perm), end(perm)));
        }
        print(cout, opt_ans_v);
        return;
    }
    // for 2
    // if (true) {
    //     vector<P> pref, suff;

    //     for (auto& [x, y] : arr) {
    //         if (abs(x - y) <= 1000) {
    //             pref.emplace_back(x, y);
    //         } else {
    //             suff.emplace_back(x, y);
    //         }
    //     }

    //     arr = pref;
    //     arr.insert(arr.end(), begin(suff), end(suff));
    // }
    // for 5
    if (n == 72018) {
        vector<P> p1, p2, p3;

        for (auto& [x, y] : arr) {
            if (x <= 5e8) {
                if (y > 5e8) {
                    p1.emplace_back(x, y);
                } else {
                    p3.emplace_back(x, y);
                }
            } else {
                p2.emplace_back(x, y);
            }
        }

        arr = p1;
        arr.insert(arr.end(), begin(p2), end(p2));
        arr.insert(arr.end(), begin(p3), end(p3));
    }

    while (clock() - c_start <= 10 * CLOCKS_PER_SEC) {
        E eval(arr);
        while (true) {
            bool opt = false;
            for (int iter = 0; iter < 3e7; iter++) {
                int u = R.rint(0, n - 1), v = R.rint(u, min(n - 1, u + 30));
                if (eval.simu_swap(u, v) <= eval.ans) {
                    eval.m_swap(u, v);
                }
                if (iter % int(1e6) == 0) {
                    // opt |= print_if_opt(eval);
                    dbg(iter, eval.ans);
                }
            }
            opt |= print_if_opt(eval);
            if (!opt) {
                break;
            }
        }
        dbg("ITER");
    }
    print(cout, opt_ans_v);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}