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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

using C = complex<long>;

template <typename A, typename B>
void add_assign(pair<A, B>& a, pair<A, B> b) {
    a.first += b.first;
    a.second += b.second;
}

ld carg(const C& p) {
    return atan2(ld(imag(p)), ld(real(p)));
}

struct cmp {
    static bool eq(const C& a, const C& b) {
        return imag(conj(a) * b) == 0;
    }

    bool operator()(const C& a, const C& b) const {
        auto to_region = [&](const C& c) -> int {
            if (imag(c) == 0) {
                assert(real(c) != 0);
                if (real(c) < 0) {
                    return 3;
                } else {
                    return 1;
                }
            } else if (imag(c) < 0) {
                return 0;
            } else {
                return 2;
            }
        };
        int ax = to_region(a), bx = to_region(b);
        if (ax != bx) {
            return ax < bx;
        }
        // TODO
        // return carg(a) < carg(b);
        return imag(conj(a) * b) > 0;
    }
};

long solve1(const vector<pair<C, long>>& arr) {
    vector<tuple<C, long, long>> mpv;
    for (auto& [p, w] : arr) {
        mpv.emplace_back(p, w, 0);
        mpv.emplace_back(-p, 0, -w);
    }
    cmp ccmp;
    sort(begin(mpv), end(mpv), [&](const auto& a, const auto& b) -> bool {
        return ccmp(get<0>(a), get<0>(b));
    });
    long ans = 0, csum = 0;
    for (auto& [p, w] : arr) {
        if (imag(p) > 0 || (imag(p) == 0 && real(p) < 0)) {
            csum += w;
        }
    }
    for (int i = 0; i < sz(mpv);) {
        int start = i;
        long ca = 0, cb = 0;
        for (; i < sz(mpv) && cmp::eq(get<0>(mpv[start]), get<0>(mpv[i])); i++) {
            ca += get<1>(mpv[i]);
            cb += get<2>(mpv[i]);
        }
        ans = max(ans, csum += ca);
        ans = max(ans, csum += cb);
    }
    return ans;
}

void solve() {
    dbg(carg(C {-1, 0}));
    int n;
    cin >> n;
    pair<C, long> arr[n];
    for (auto& [p, w] : arr) {
        long x, y;
        cin >> x >> y >> w;
        p = {x, y};
    }
    long ans = 0;
    for (int i = 0; i < n; i++) {
        vector<pair<C, long>> carr;
        for (int j = 0; j < n; j++) {
            if (i == j) {
                continue;
            }
            carr.emplace_back(arr[j].first - arr[i].first, arr[j].second);
        }
        ans = max(ans, arr[i].second + solve1(carr));
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}