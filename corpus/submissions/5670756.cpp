#include <algorithm>
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

struct R {
    mt19937_64 cowng =
        mt19937_64(chrono::steady_clock::now().time_since_epoch().count());

    long rint(long l, long r) {
        return cowng() % (r - l + 1) + l;
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
    auto f = [&](ld x) -> ld { return abs(shrimp(x) - pos) - speed * (x - t); };

    ld l = t, r = ternary_search(l, 1e9, f);
    if (f(r) > 0) {
        return 1e9;
    }

    while (r - l > 1e-6) {
        ld mid = (l + r) / 2;
        if (f(mid) <= 0) {
            r = mid;
        } else {
            l = mid;
        }
    }

    return l;
}

struct Ans {
    ld w_sum;
    vector<pair<ld, Shrimp>> arr;
};

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
        ld nt = chase(t, s, pos, speed) + 1e-5;
        if (nt > max_t) {
            continue;
        }

        t = nt;
        pos = s(t);

        w += s.w;
        out_ans.emplace_back(nt, s);
    }

    return {w - ic.w, out_ans};
}

void solve() {
    dbg(chase(0, {0, {0, 10}, {0, 1}, 0}, {0, 0}, 10));

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

    // arr.erase(remove_if(begin(arr), end(arr),
    //                     [&](const Shrimp& s) -> bool {
    //                         return norm(s.vel) >= ic.speed * ic.speed;
    //                     }),
    //           arr.end());

    sort(begin(arr), end(arr),
         CmpByKey([&](const auto& s) -> ld { return abs(s.pos0 - ic.pos); }));
    stable_sort(begin(arr), end(arr),
                CmpByKey([&](const auto& s) -> ld { return s.w; }));
    if (tc_id == 4) {
        int i = 0;
        for (; arr[i].w < 100; i++)
            ;
        sort(arr.begin() + i, arr.end(), CmpByKey([&](const auto& s) -> ld {
                 if (s.pos0.real() > ic.pos.real()) {
                     return 1e9;
                 }
                 return abs(s.pos0 - ic.pos);
             }));
    }

    ld c_opt = 0;

    // int ord[n];
    // iota(ord, ord + n, 0);

    // do {
    //     vector<Shrimp> carr(n);
    //     for (int i = 0; i < n; i++) {
    //         carr[ord[i]] = arr[i];
    //     }
    //     print_if_opt(eval(ic, carr));
    // } while (next_permutation(ord, ord + n));

    Timer timer;
    Ans ans;

    while (timer.elapsed_secs() < 20) {
        int u = R.rint(0, n - 1), v = R.rint(0, n - 1);
        swap(arr[u], arr[v]);

        auto nans = eval(ic, arr);
        if (nans.w_sum >= c_opt) {
            c_opt = nans.w_sum;
            ans = nans;
            // print_if_opt(nans);
        } else {
            swap(arr[u], arr[v]);
        }
    }

    print(cout, ans);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}