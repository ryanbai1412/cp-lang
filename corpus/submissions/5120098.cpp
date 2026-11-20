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

bool query(int l, int r) {
    cout << "Look " << l << " " << r << endl;
    string ans;
    cin >> ans;
    return ans == "True";
}

int answer(int v, int p) {
    cout << "Guess " << v << " " << p << endl;
}

void solve() {
    cout << "GetSize" << endl;
    int n;
    cin >> n;
    mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
    int x;
    do {
        x = cowng() % n;
    } while (!query(x, x));
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}