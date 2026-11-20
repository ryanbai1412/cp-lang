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
#define sz(x) int((x).size())

constexpr int dirs[8][2] {{-1, -1}, {-1, 0}, {-1, 1}, {0, -1},
                          {0, 1},   {1, -1}, {1, 0},  {1, 1}};

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());
    auto rint = [&](long l, long r) -> long {
        return cowng() % (r - l + 1) + l;
    };
    auto rfloat = [&](double l, double r) -> double {
        return uniform_real_distribution<double>(l, r)(cowng);
    };

//    int n = rint(1, 10), q = 1000, m = rint(3, 1e7), b = rint(2, m - 1);
//    int n = 2e5, q = 5e5, m = rint(3, 1e7), b = rint(2, m - 1);
    int n = 10, q = 10000, m = rint(3, 1e7), b = rint(2, m - 1);
    cout << n << " " << q << " " << m << " " << b << endl;
    for (int i = 0; i < n; i++) {
        cout << rint(0, m - 1) << " \n"[i == n - 1];
    }
    while (q--) {
        int l = rint(1, n), r = rint(l, n);
        cout << rint(0, 1) << " " << l << " " << r << endl;
    }
}