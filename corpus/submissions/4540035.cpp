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
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int q;
	cin >> q;
	while (q--) {
		int m;
		cin >> m;
		int carr[m];
		for (auto& a : carr) {
			cin >> a;
		}
		int ans = 0;
		for (int i = 0; i < (1 << n); i++) {
			for (auto& a : carr) {
				bool ok = false;
				for (int j = 0; j < n; j++) {
					if ((i >> j) & 1) {
						ok |= arr[j] % a == 0;
					}
				}
				if (!ok) {
					goto loop;
				}
			}
			ans++;
		loop:;
		}
		cout << ans << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}