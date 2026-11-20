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
		if (n == 1) {
			cout << "1" << endl;
			cout << "1 1" << endl;
		} else if (n == 2) {
			cout << "2" << endl;
			int x, y;
			cin >> x >> y;
			cout << (1 ^ 2 ^ x) << " " << (1 ^ 2 ^ y) << endl;
		} else {
			cout << "1" << endl;
			cout << "2 2" << endl;
			set<pair<int, int>> vis;
			for (int a: {1, 3}) {
				for (int b: {1, 3}) {
					vis.emplace(a, b);
				}
			}
			while (sz(vis)) {
				int x, y;
				cin >> x >> y;
				vis.erase({x, y});
				auto it = vis.begin();
				cout << it->first << " " << it->second << endl;
				vis.erase(it);
			}
		}
		int a, b;
		cin >> a >> b;
		assert(!a && !b);
	}
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}