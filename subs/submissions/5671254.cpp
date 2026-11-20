#include <random>
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

using ld = long double;
using P = complex<ld>;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

template <typename T>
T sq(T x) {
    return x * x;
}

struct R {
    mt19937_64 cowng =
        mt19937_64(chrono::steady_clock::now().time_since_epoch().count());

    long rint(long l, long r) {
        return cowng() % (r - l + 1) + l;
    }

    ld rfloat(ld l, ld r) {
        return uniform_real_distribution<>(l, r)(cowng);
    }
} R;

struct Timer {
    clock_t c_start = clock();

    ld elapsed_secs() {
        return ld(clock() - c_start) / CLOCKS_PER_SEC;
    }
};

struct InitConfig {
    ld w, speed, max_t;
    P pos;
};

struct Shrimp {
    ld w;
    P pos0, vel;
    int id;

    P operator()(ld t) const {
        return pos0 + t * vel;
    }

    bool operator==(const Shrimp& s) const {
        return id == s.id;
    }
};

template <typename F>
ld ternary_search(ld l, ld r, const F& f) {
    while (r - l > 1e-6 && l < 1e9) {
        ld diff = (r - l) / 3, m1 = l + diff, m2 = m1 + diff;
        if (f(m1) < f(m2)) {
            r = m2;
        } else {
            l = m1;
        }
    }

    return l;
}

ld chase(ld t, const Shrimp& shrimp, P pos, ld speed) {
    auto f = [&](ld x) -> ld {
        return norm(shrimp(x) - pos) - sq(speed * (x - t));
    };

    ld l = t, r = ternary_search(l, 1e5, f);
    if (f(r) > -1e-5) {
        return 1e9;
    }

    while (r - l > 1e-6) {
        ld mid = (l + r) / 2;
        if (f(mid) <= -1e-5) {
            r = mid;
        } else {
            l = mid;
        }
    }

    assert(f(r) <= 0);
    return r;
}

struct Ans {
    ld w_sum;
    vector<pair<ld, Shrimp>> arr;

    Ans() : w_sum(0) {}
    Ans(const vector<pair<ld, Shrimp>>& arr) : w_sum(0), arr(arr) {
        for (auto& a : arr) {
            w_sum += a.second.w;
        }
    }
    Ans(ld w_sum, const vector<pair<ld, Shrimp>>& arr)
        : w_sum(w_sum), arr(arr) {}
};

void check(const InitConfig& ic, const Ans& ans) {
    ld t = 0;
    P pos = ic.pos;
    for (auto& [ct, s] : ans.arr) {
        assert(t <= ct);
        assert(abs(s(ct) - pos) <= ic.speed * (ct - t));
        pos = s(ct);
    }
}

void print(ostream& out, const Ans& ans) {
    out << fixed << setprecision(10);

    out << sz(ans.arr) << endl;
    out << ans.w_sum << endl;
    for (auto& [t, s] : ans.arr) {
        auto pos = s(t);
        out << t << " " << real(pos) << " " << imag(pos) << " " << s.id << endl;
    }
}

ld opt_w_sum;

void print_if_opt(const Ans& ans) {
    if (ans.w_sum <= opt_w_sum) {
        return;
    }
    opt_w_sum = ans.w_sum;
    dbg(ans.w_sum);
    ofstream out("output.txt");
    print(out, ans);
}

Ans eval(const InitConfig& ic, const vector<Shrimp>& arr) {
    auto [w, speed, max_t, pos] = ic;

    vector<pair<ld, Shrimp>> out_ans;
    ld t = 0;
    for (auto& s : arr) {
        if (s.w >= w) {
            continue;
        }
        ld nt = chase(t, s, pos, speed);
        if (nt > max_t) {
            continue;
        }

        t = nt;
        pos = s(t);

        w += s.w;
        out_ans.emplace_back(nt, s);
    }

    Ans ans {w - ic.w, out_ans};
    check(ic, ans);
    return ans;
}

vector<int> solve5_1(const vector<tuple<ld, ld, ld>>& arr) {
    int n = sz(arr);
    pair<ld, int> dp[n] {};
    for (int i = n - 1; i >= 0; i--) {
        dp[i] = {0, -1};
        auto& [w, t, pos] = arr[i];
        for (int j = i + 1; j < n; j++) {
            auto& [nw, nt, npos] = arr[j];
            assert(t <= nt);
            if (abs(npos - pos) <= 0.05 * (nt - t)) {
                dp[i] = max(dp[i], {dp[j].first, j});
            }
        }
        dp[i].first += w;
    }

    vector<int> ans;
    int u = 0;
    while (u != -1) {
        ans.push_back(u);
        u = dp[u].second;
    }
    return ans;
}

void solve5(const InitConfig& ic, vector<Shrimp> arr) {
    int n = sz(arr);

    assert(ic.pos == P(0, 0));

    for (auto& a : arr) {
        assert((a.vel == P {0, -1000}));
    }
    sort(begin(arr), end(arr),
         CmpByKey([&](const Shrimp& s) -> ld { return s.pos0.imag(); }));

    bool use[n] {};
    {
        vector<tuple<ld, ld, ld>> n_arr {{0, 0, 0}};
        for (auto& a : arr) {
            n_arr.emplace_back(a.w, a.pos0.imag() / 1000, a.pos0.real());
        }
        auto cans = solve5_1(n_arr);

        for (auto& a : cans) {
            use[a - 1] = true;
        }
    }

    P pos = ic.pos;
    ld t = 0;
    vector<pair<ld, Shrimp>> ans_arr;

    dbg(ic.speed);
    for (int i = 0; i < n; i++) {
        auto& a = arr[i];
        ld nt = a.pos0.imag() / 1000;
        if (use[i]) {
            dbg(pos, a(nt), t, nt);
            assert(abs(a(nt) - pos) <= ic.speed * (nt - t) + 1e-4);
            ans_arr.emplace_back(nt, a);
            t = nt;
            pos = a(nt);
        }
    }

    Ans ans = ans_arr;
    print_if_opt(ans);
}

// this is so troll
void solve8(const InitConfig& ic, vector<Shrimp> arr) {
    for (auto& a : arr) {
        assert(a.vel == P());
    }

    constexpr int n = 100, m = 2000;

    long arr_w[n][m] {};
    int arr_id[n][m];
    memset(arr_id, -1, sizeof(arr_id));

    for (auto& a : arr) {
        int x = a.pos0.imag(), y = a.pos0.real();
        arr_w[x][y] = a.w;
        assert(arr_id[x][y] == -1);
        arr_id[x][y] = a.id;
    }

    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < m; j++) {
            if (i == 60 || arr_id[i][j] == -1) {
                continue;
            }
            dbg(i, j);
            assert(arr_id[i + 1][j + 1] != -1 && arr_id[i + 1][j - 1] != -1 &&
                   arr_id[i + 1][j] == -1);
        }
    }

    long psum = 0;
    for (int i = 0; i < 60; i++) {
        long cmn = LLONG_MAX, cmx = *max_element(arr_w[i], arr_w[i] + m);
        for (auto& a : arr_w[i]) {
            if (a) {
                cmn = min(cmn, a);
            }
        }
        psum += cmx;
        dbg(psum, cmn, cmx);
    }

    long dp[n][m] {};
    for (int i = n - 2; i >= 0; i--) {
        for (int j = 1; j < m - 1; j++) {
            dp[i][j] = max(dp[i + 1][j - 1], dp[i + 1][j + 1]) + arr_w[i][j];
        }
    }

    vector<pair<ld, Shrimp>> ans_arr {{1, arr[0]}};

    ld t = 1;
    int x = 1, y = 1000;
    while (true) {
        if (x == 60) {
            break;
        }
        if (dp[x + 1][y - 1] + arr_w[x][y] == dp[x][y]) {
            y--;
        } else {
            y++;
        }
        x++;
        t += sqrt(2);
        ans_arr.emplace_back(t, arr[arr_id[x][y]]);
    }

    Ans ans = ans_arr;
    dbg(ans.w_sum, dp[1][1000]);
    print_if_opt(ans);
}

void solve() {
    int tc_id;
    cin >> tc_id;

    InitConfig ic;
    {
        ld w0, speed, max_t, x0, y0;
        cin >> w0 >> speed >> max_t >> x0 >> y0;
        ic = {w0, speed, max_t, {x0, y0}};
    }
    assert(ic.max_t <= 1e9);

    int n;
    cin >> n;
    vector<Shrimp> arr(n);
    for (int i = 0; i < n; i++) {
        auto& [w, pos0, vel, id] = arr[i];
        id = i + 1;
        ld px0, py0, vx, vy;
        cin >> w >> px0 >> py0 >> vx >> vy;
        pos0 = {px0, py0};
        vel = {vx, vy};
    }

    if (tc_id == 8) {
        solve8(ic, arr);
        return;
    }

    // arr.erase(remove_if(begin(arr), end(arr),
    //                     [&](const Shrimp& s) -> bool {
    //                         return norm(s.vel) >= ic.speed * ic.speed;
    //                     }),
    //           arr.end());

    {
        sort(begin(arr), end(arr),
             CmpByKey([&](const auto& s) -> ld { return -s.w; }));
        for (int i = 0; i < 10; i++) {
            dbg(arr[i].w, arr[i].pos0);
        }
        for (int i = n - 10; i < n; i++) {
            dbg(arr[i].w, arr[i].pos0);
        }
    }

    sort(begin(arr), end(arr),
         CmpByKey([&](const auto& s) -> ld { return abs(s.pos0 - ic.pos); }));
    if (tc_id != 4 && tc_id != 5) {
        stable_sort(begin(arr), end(arr),
                    CmpByKey([&](const auto& s) -> ld { return s.w; }));
    }

    if (tc_id == 4) {
        int i = 0;
        for (int x = 6; x < 25165824; x *= 2, i++) {
            swap(*find_if(begin(arr), end(arr),
                          [&](const Shrimp& s) -> bool { return s.w == x; }),
                 arr[i]);
        }
        sort(arr.begin() + i, arr.end(), CmpByKey([&](const auto& s) -> ld {
                 if (s.pos0.real() < 0) {
                     return 1e9;
                 }
                 return abs(s.pos0 - ic.pos);
             }));
    }
    if (tc_id == 5 || tc_id == 6) {
        solve5(ic, arr);
        return;
    }
    if (tc_id == 9) {
        P pos {};
        ld cw = 1000;

        for (int i = 0; i < n; i++) {
            pair<ld, int> opt {1e18, -1};
            for (int j = i; j < n; j++) {
                if (arr[j].w < cw) {
                    opt = min(opt, {abs(arr[j].pos0 - pos), j});
                }
            }

            assert(opt.second != -1);
            swap(arr[i], arr[opt.second]);
            cw += arr[i].w;
            pos = arr[i].pos0;
        }
    }
    if (tc_id == 10) {
        ld c_bord_v = 0.0005688001;

        while (true) {
            auto [w, speed, max_t, pos] = ic;

            ld t = 0;
            for (int i = 0; i < n; i++) {
                ld bord = c_bord_v;
                ld copt_t = 1e18;
                vector<pair<ld, int>> vals;

                for (int j = i; j < n; j++) {
                    auto& s = arr[j];
                    if (s.w >= w || abs(s(copt_t + bord) - pos) >
                                        (copt_t + bord - t) * speed) {
                        continue;
                    }
                    ld nt = chase(t, s, pos, speed);
                    copt_t = min(copt_t, nt);

                    vals.emplace_back(nt, j);
                }

                sort(begin(vals), end(vals));
                while (vals.back().first > vals[0].first + bord) {
                    vals.pop_back();
                }

                pair<ld, pair<ld, int>> opt2 {1e18, {}};
                for (auto& p : vals) {
                    int ind = p.second;
                    ld ans = 0;

                    for (int j = i; j < n; j++) {
                        ans += abs(arr[ind](t) - arr[j](t));
                    }

                    opt2 = min(opt2, {ans, p});
                }

                // auto [nt, ni] = *min_element(begin(vals), end(vals));
                auto [nt, ni] = opt2.second;
                assert(ni != -1);
                swap(arr[i], arr[ni]);
                w += arr[i].w;
                t = nt;
                pos = arr[i](t);

                if (i % 100 == 0) {
                    print_if_opt(eval(ic, arr));
                }
            }
            Ans cans = eval(ic, arr);
            print_if_opt(cans);

            dbg(c_bord_v, cans.w_sum);
            c_bord_v *= 0.99;
        }
    }
    dbg("READY", opt_w_sum);

    Timer timer;

    Ans ans {};
    int max_swap;

    auto try_do_swap = [&](int u, int v) -> void {
        swap(arr[u], arr[v]);

        auto nans = eval(ic, arr);
        if (nans.w_sum >= ans.w_sum) {
            ans = nans;
            max_swap = int(find(begin(arr), end(arr), nans.arr.back().second) -
                           arr.begin());
#ifdef LOCAL
            print_if_opt(nans);
#endif
        } else {
            swap(arr[u], arr[v]);
        }
    };

    try_do_swap(0, 0);
    dbg("A");

    int iters = 0;
#ifdef LOCAL
    while (true) {
#else
    while (timer.elapsed_secs() < 1) {
#endif
        int l = R.rint(0, n - 1), r = R.rint(l, min(n - 1, l + 100));
        if (!R.rint(0, 100)) {
            r = n - 1;
        }
        try_do_swap(l, r);
    }

    print(cout, ans);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cerr << fixed << setprecision(10);

    solve();
}