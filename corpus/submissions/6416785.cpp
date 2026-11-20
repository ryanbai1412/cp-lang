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
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

constexpr int MAXN = 2005;

bool graph[MAXN][MAXN];

void check(const vector<int>& arr) {
    int n = sz(arr), cnt = 0;

    for (int i = 0; i + 2 < n; i++) {
        cnt += graph[arr[i]][arr[i + 1]] != graph[arr[i + 1]][arr[i + 2]];
    }

    assert(cnt <= 1);
}

vector<int> solve(const vector<int>& ord) {
    deque<int> blue, red;

    if (graph[ord[0]][ord[1]]) {
        red.push_back(ord[0]);
        red.push_back(ord[1]);
    } else {
        blue.push_back(ord[0]);
        blue.push_back(ord[1]);
    }

    for (int oi = 2; oi < sz(ord); oi++) {
        int u = ord[oi];

        if (!sz(red)) {
            int v = blue.back();

            if (graph[v][u]) {  // 1
                blue.pop_back();
                red.push_front(v);
                if (graph[u][blue.front()]) {
                    red.push_back(u);
                } else {
                    blue.push_front(u);
                }
            } else {  // 2
                if (graph[u][blue.front()]) {
                    red.push_back(u);
                } else {
                    blue.push_front(u);
                }
            }
        } else {
            int v = red.front();

            if (graph[u][v]) {  // 3
                red.push_front(u);

                if (sz(blue)) {
                    if (graph[blue.back()][u]) {
                        red.push_front(blue.back());
                        blue.pop_back();
                    }
                } else {
                    if (!graph[red.back()][u]) {
                        blue.push_back(red.back());
                        red.pop_back();
                    }
                }
            } else {  // 4
                blue.push_back(v);
                red.pop_front();

                int nxt = sz(red) ? red.front() : blue.front();
                if (graph[u][nxt]) {
                    red.push_front(u);
                } else {
                    blue.push_back(u);
                }
            }
        }
    }

    dbg(vector(begin(blue), end(blue)), vector(begin(red), end(red)));
    vector<int> vals(begin(blue), end(blue));
    vals.insert(end(vals), begin(red), end(red));

    int u = ord.back();
    if ((sz(blue) && blue.back() == u) || (sz(red) && red.back() == u)) {
        reverse(begin(vals), end(vals));
    }

    rotate(begin(vals), find(begin(vals), end(vals), ord.back()), end(vals));

    return vals;
}

void solve() {
    int n;
    cin >> n;
    for (int i = 1; i < n; i++) {
        string s;
        cin >> s;
        for (int j = 0; j < i; j++) {
            graph[i][j] = graph[j][i] = s[j] == 'R';
        }
    }

    vector<int> ord(n);
    iota(begin(ord), end(ord), 0);
    for (int i = 0; i < n; i++) {
        rotate(begin(ord), begin(ord) + 1, end(ord));

        auto ans = solve(ord);

        dbg(i, ans);
        assert(ans[0] == i && sz(ans) == n);
        check(ans);

        cout << n << endl;
        for (int j = 0; j < n; j++) {
            cout << ans[j] + 1 << " \n"[j + 1 == n];
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}