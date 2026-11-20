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

void solve() {
    int a, b, c, d;
    cin >> a >> b >> c >> d;
    auto f = [&](double x) -> double {
        return a * x * x * x + b * x * x + c * x + d;
    };
    vector<double> ans;
    for (double x = -105; x <= 105; x += .001) {
        double fl = f(x), fr = f(x + .001);
        if (fl * fr < 0 || fl == 0) {
            ans.push_back(x);
        }
    }
    for (int i = 0; i < 3; i++) {
        cout << ans[i] << " \n"[i == 2];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}