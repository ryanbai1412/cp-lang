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

void solve() {
	int n, t;
	cin >> n >> t;
	while (t--) {
		bool diag[n] {};
		if (n & 1) {
			cout << "1" << endl;
			cout << "1 " << n << endl;
			diag[0] = true;
		} else {
			cout << "2" << endl;
		}
		while (true) {
			int x, y;
			cin >> x >> y;
			assert(x != -1 && y != -1);
			if (!x && !y) {
				break;
			}
			if (x + y == n + 1) {
				diag[x - 1] = true;
				for (int i = 0; i < n; i++) {
					if (!diag[i]) {
						diag[i] = true;
						cout << i + 1 << " " << n - i << endl;
					}
				}
			} else {
				cout << n + 1 - x << " " << n + 1 - y << endl;
			}
		}
	}
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}