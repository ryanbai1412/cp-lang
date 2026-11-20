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

constexpr int maxn = 10;

struct City {
    double dist, cost;
} arr[maxn];

int n;
double cap, ans;

void dfs(int i, double gas, double cost) {
    if (i == n) {
        ans = min(ans, cost);
        return;
    }
    double cnxt = gas, cadd = 0;
    for (int j = i; j < n; j++) {
        cadd += arr[j].dist;
        cadd = min(cadd, cap);
        cnxt = max(cnxt, cadd);
        dfs(i + 1, cnxt - arr[i].dist, cost + arr[i].cost * (cnxt - gas));
    }
}

void solve() {
    double tdist, mpg;
    cin >> tdist >> cap >> mpg >> arr[0].cost >> n;
    n++;
    double pos[n + 1];
    pos[0] = 0;
    pos[n] = tdist;
    for (int i = 1; i < n; i++) {
        cin >> pos[i] >> arr[i].cost;
    }
    for (int i = 0; i < n; i++) {
        arr[i].dist = pos[i + 1] - pos[i];
        arr[i].cost /= mpg;
    }
    cap *= mpg;
    for (int i = 0; i < n; i++) {
        if (arr[i].dist > cap) {
            cout << "No Solution" << endl;
            return;
        }
    }
    ans = INFINITY;
    dfs(0, 0, 0);
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}