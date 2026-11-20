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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

void solve() {
	int n, m;
	long k;
	cin >> n >> m >> k;
	if (k < n) {
		cout << -1 << endl;
		return;
	}
	int ans[n];
	long cans = k;
	for (int i = 0; i < n; i++) {
		long cadd = min(i + 1, m), additional = n - i - 1;
		if (cadd + additional > cans) {
			cadd = cans - additional;
			ans[i] = ans[i - cadd];
		} else {
			ans[i] = i % m;
		}
		cans -= cadd;
	}
	assert(cans >= 0);
	if (cans) {
		cout << -1 << endl;
	} else {
		for (int i = 0; i < n; i++) {
			cout << ans[i] + 1 << " \n"[i == n - 1];
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}