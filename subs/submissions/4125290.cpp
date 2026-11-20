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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template<int maxn>
struct pfact {
	int mdiv[maxn + 1];
	
	pfact(): mdiv {} {
		for (int i = 2; i <= maxn; ++i) {
			if (mdiv[i] == 0) {
				mdiv[i] = i;
				if (i <= maxn / i) {
					for (int j = i * i; j <= maxn; j += i) {
						if (mdiv[j] == 0) {
							mdiv[j] = i;
						}
					}
				}
			}
		}
	}
	
	template<typename T>
	void fact(int x, int cmul, const T &f) {
		if (x == 1) {
			f(cmul);
		} else {
			int cnt = 0, cur = mdiv[x];
			while (mdiv[x] == cur) {
				x /= cur;
				cnt++;
			}
			fact(x, cmul, f);
			for (int i = 0; i < cnt; i++) {
				fact(x, cmul *= cur, f);
			}
		}
	}
};

const int maxn = 5e6 + 5;

pfact<maxn> f;

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	sort(arr, arr + n);
	bool vis[maxn] {};
	int ans = 0;
	for (auto& a : arr) {
		bool cur = false;
		f.fact(a, 1, [&](int x) {
			if (vis[x]) {
				cur = true;
			}
		});
		if (!cur) {
			ans++;
			vis[a] = true;
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}