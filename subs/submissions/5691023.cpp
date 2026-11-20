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
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct Cmp {
    Cb cb;

    Cmp(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

using P = complex<long>;

bool ccw(P p1, P p2) {
    return imag(p1 * conj(p2)) <= 0;
}

bool ccw(P p1, P p2, P p3) {
    return ccw(p2 - p1, p3 - p2);
}

vector<pair<P, int>> ch_arr(vector<P> m_poly) {
    long mn_x = 1e18, mx_x = -1e18;
    for (auto& a : m_poly) {
        mn_x = min(mn_x, real(a));
        mx_x = max(mx_x, real(a));
    }

    if (mn_x == mx_x) {
        return {{}};
    }

    vector<pair<P, int>> poly;
    for (int i = 0; i < sz(m_poly); i++) {
        poly.emplace_back(m_poly[i], i);
    }

    sort(begin(poly), end(poly),
         Cmp([&](const pair<P, int>& p) -> pair<long, long> {
             return {real(p.first), -imag(p.first)};
         }));

    vector<pair<P, int>> arr = poly;
    while (real(arr.back().first) == mx_x) {
        arr.pop_back();
    }

    arr.insert(arr.end(), poly.rbegin(), poly.rend());
    while (real(arr.back().first) == mn_x) {
        arr.pop_back();
    }
    arr.push_back(arr[0]);

    return arr;
}

vector<pair<P, int>> convex_hull(vector<P> poly) {
    auto arr = ch_arr(poly);

    vector<pair<P, int>> st;
    for (int i = 0; i < sz(arr); i++) {
        while (sz(st) >= 2 &&
               !ccw(end(st)[-2].first, end(st)[-1].first, arr[i].first)) {
            st.pop_back();
        }
        st.emplace_back(arr[i].first, i);
    }

    return st;
}

long area_h(P p1, P p2) {
    return (imag(p1) + imag(p2)) * (real(p1) - real(p2));
}

long hull_area(const vector<pair<P, int>>& hull) {
    long ans = 0;
    for (int i = 0; i < sz(hull); i++) {
        int ni = (i + 1) % sz(hull);
        ans += area_h(hull[i].first, hull[ni].first);
    }
    return ans;
}

long polygon_area(const vector<P>& poly) {
    return hull_area(convex_hull(poly));
}

void solve() {
    int n, q;
    cin >> n >> q;

    vector<P> poly(n);
    for (auto& a : poly) {
        long x, y;
        cin >> x >> y;
        a = {x, y};
    }

    auto solve_brute = [&](const vector<int>& carr) -> long {
        bool i_carr[n] {};
        for (auto& a : carr) {
            i_carr[a] = true;
        }
        vector<P> narr;
        for (int i = 0; i < n; i++) {
            if (!i_carr[i]) {
                narr.push_back(poly[i]);
            }
        }

        return polygon_area(narr);
    };

    auto arr = ch_arr(poly);
    auto hull = convex_hull(poly);
    dbg(arr, hull);

    vector<int> i_hull[n];
    for (int i = 0; i < sz(hull); i++) {
        i_hull[arr[hull[i].second].second].push_back(i);
    }

    long last_ans = -1, base_ans = hull_area(hull);

    map<int, long> memo;

    while (q--) {
        int m;
        cin >> m;
        assert(m == 1);

        vector<int> carr(m);
        for (auto& a : carr) {
            cin >> a;
            a = (last_ans + a) % n;
        }
        int ind = carr[0];

        auto [it, inserted] = memo.emplace(ind, -1);
        long& ans = it->second;
        if (!inserted) {
            cout << (last_ans = ans) << endl;
            continue;
        }

        if (!sz(i_hull[ind])) {
            cout << (ans = last_ans = base_ans) << endl;
            continue;
        } else if (i_hull[ind][0] < 2 || i_hull[ind].back() >= sz(hull) - 2) {
            cout << (ans = last_ans = solve_brute(carr)) << endl;
            continue;
        }

        auto go = [&](int s_ind) -> long {
            dbg(s_ind);
            long ans = 0;
            ans -= area_h(hull[s_ind - 1].first, hull[s_ind].first);
            ans -= area_h(hull[s_ind].first, hull[s_ind + 1].first);

            vector<P> st {hull[s_ind - 2].first, hull[s_ind - 1].first};
            for (int i = hull[s_ind - 1].second + 1;
                 i <= hull[s_ind + 1].second; i++) {
                if (arr[i].second == ind) {
                    continue;
                }
                while (sz(st) >= 2 &&
                       !ccw(end(st)[-2], end(st)[-1], arr[i].first)) {
                    st.pop_back();
                }
                st.push_back(arr[i].first);
            }

            dbg(st);
            for (int i = 1; i < sz(st) - 1; i++) {
                ans += area_h(st[i], st[i + 1]);
            }
            return ans;
        };

        ans = base_ans;
        for (auto& a : i_hull[ind]) {
            ans += go(a);
        }

        cout << (last_ans = ans) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}