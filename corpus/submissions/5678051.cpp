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

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

struct P {
    int x, y;

    friend ostream& operator<<(ostream& out, const P& p) {
        return out << "(" << p.x << ", " << p.y << ")";
    }

    bool operator<(const P& p) const {
        return x < p.x || (x == p.x && y < p.y);
    }

    bool operator==(const P& p) const {
        return x == p.x && y == p.y;
    }

    bool operator!=(const P& p) const {
        return !(*this == p);
    }

    P& operator+=(const P& p) {
        x += p.x;
        y += p.y;
        return *this;
    }

    P& operator-=(const P& p) {
        x -= p.x;
        y -= p.y;
        return *this;
    }

    friend P operator+(P a, const P& b) {
        return a += b;
    }
    friend P operator-(P a, const P& b) {
        return a -= b;
    }

    static P from_dir(char c) {
        if (c == '<') {
            return {0, -1};
        } else if (c == '>') {
            return {0, +1};
        } else if (c == '^') {
            return {-1, 0};
        } else if (c == 'v') {
            return {+1, 0};
        }
        dbg(c);
        assert(false);
    }
};

P simu(int n, int m, P bad, const string& s) {
    auto ibs = [&](P p) -> bool {
        return 0 <= p.x && p.x < n && 0 <= p.y && p.y < m;
    };
    P pos {0, 0};
    for (auto& a : s) {
        P npos = pos + P::from_dir(a);
        if (ibs(npos) && npos != bad) {
            pos = npos;
        }
    }
    return pos;
}

string repeat(const string& s, int n) {
    string ans;
    for (int i = 0; i < n; i++) {
        ans += s;
    }
    return ans;
}

string gen1(int n, int m) {
    string ans;
    for (int i = 1; i < n; i++) {
        if (i & 1) {
            ans += repeat("v>^", m - 1);
            ans += ">v>";
        } else {
            ans += repeat("v<^", m - 1);
            ans += "<v<";
        }
    }
    return ans;
}

bool solves_c(int n, int m, const vector<P>& arr, const string& s) {
    set<P> cs;
    for (auto& a : arr) {
        if (!cs.insert(simu(n, m, a, s)).second) {
            return false;
        }
    }
    return true;
}

string gen2(int n, int m, int ind) {
    return repeat("v", ind) + repeat(">", m);
}

string gen3(int n, int m) {
    return ">vv<^v" + repeat(">", m) + "^v" + repeat("<", m) + "^^" +
           repeat(">", m);
}

int n, m;
#ifdef LOCAL
P g_ans;
#endif

P query(const string& s) {
#ifdef LOCAL
    return simu(n, m, g_ans, s);
#endif
    cout << "? " << s << endl;
    int x, y;
    cin >> x >> y;
    return {x, y};
}

void answer(P p) {
#ifdef LOCAL
    assert(g_ans == p);
#endif
    auto& [x, y] = p;
    cout << "! " << x << " " << y << endl;
}

vector<P> query_filt(int n, int m, const string& s, const vector<P>& cands) {
    P res = query(s);
    vector<P> ans;
    for (auto& a : cands) {
        if (simu(n, m, a, s) == res) {
            ans.push_back(a);
        }
    }
    return ans;
}

void solve() {
    cin >> n >> m;
#ifdef LOCAL
    cin >> g_ans.x >> g_ans.y;
#endif
    vector<P> cands;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (i || j) {
                cands.push_back({i, j});
            }
        }
    }
    if (n == 1) {
        answer(query_filt(n, m, repeat(">", m), cands)[0]);
        return;
    }
    cands = query_filt(n, m, gen1(n, m), cands);

    vector<string> strats;
    for (int i = 0; i < n; i++) {
        strats.push_back(gen2(n, m, i));
    }
    strats.push_back(gen3(n, m));

    for (auto& a : strats) {
        if (solves_c(n, m, cands, a)) {
            cands = query_filt(n, m, a, cands);
            assert(sz(cands) == 1);
            answer(cands[0]);
            return;
        }
    }

    assert(false);
}

int main() {
    // cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}