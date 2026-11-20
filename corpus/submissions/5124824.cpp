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

void empty(int x) {
    cout << "E " << x + 1 << endl;
}

void fill(int x) {
    cout << "F " << x + 1 << endl;
}

int pour(int x, int y) {
    cout << "D " << x + 1 << " " << y + 1 << endl;
    int ans;
    cin >> ans;
    return ans;
}

void solve() {
    int n, m;
    cin >> n >> m;
    for (int i = 0; i < n; i++) {
        empty(i);
    }
    int arr[n] {};
    while (true) {
        int x = pour(1, 0);
        if (!x) {
            break;
        }
        arr[0] += x;
    }
    int pi = 0;
    for (int i = 1; i < n; i++) {
        int x = pour(pi, i);
        if (x < arr[pi]) {
            arr[i] = x;
            fill(pi);
            continue;
        }
        while (x) {
            arr[i] += x;
            fill(pi);
            x = pour(pi, i);
        }
        pi = i;
    }
    cout << "A";
    for (int i = 0; i < n; i++) {
        cout << " " << arr[i];
    }
    cout << endl;
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}