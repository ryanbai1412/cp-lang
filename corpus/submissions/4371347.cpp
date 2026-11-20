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
	int n, c;
	cin >> n >> c;
	long arr[c * 2] {};
	for (int i = 0; i < n; i++) {
		int x;
		cin >> x;
		arr[x]++;
	}
	long psum[c * 2 + 1];
	psum[0] = 0;
	for (int i = 0; i < c * 2; i++) {
		psum[i + 1] = psum[i] + arr[i];
	}
	long psum2[c * 2 + 1], arr2[c * 2];
	psum2[0] = 0;
	auto gt = [&](int i) -> long {
		return i < c * 2 ? psum[i] : 0;
	};
	for (int i = 0; i < c * 2; i++) {
		arr2[i] = arr[i] * gt(i + (c + 1) / 2);
		psum2[i + 1] = psum2[i] + arr2[i];
	}
	long ans = 0;
	for (int i = 0; i < c; i++) {
		long csm = psum2[i + (c + 1) / 2] - psum2[i + 1];
		csm -= (psum[i + (c + 1) / 2] - psum[i + 1]) * psum[i + c / 2 + 1];
		ans += csm * arr[i];
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}