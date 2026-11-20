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

const int maxn = 1e4 + 5, maxb = maxn * maxn;

bitset<maxb> nprime;

void pcomp() {
    nprime[0] = nprime[1] = true;
	for (int i = 2; i < maxb; i++) {
		if (i <= maxb / i && !nprime[i]) {
			for (int j = i * i; j < maxb; j += i) {
				nprime[j] = true;
			}
		}
	}
}

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int psum[n + 1];
	psum[0] = 0;
	for (int i = 0; i < n; i++) {
		psum[i + 1] = psum[i] + arr[i];
	}
	for (int i = 2; i <= n; i++) {
		for (int j = 0; j + i <= n; j++) {
			if (!nprime[psum[i + j] - psum[j]]) {
				cout << "Shortest primed subsequence is length " << i << ":";
				for (int k = j; k < j + i; k++) {
					cout << " " << arr[k];
				}
				cout << endl;
				return;
			}
		}
	}
	cout << "This sequence is anti-primed." << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	int tcs;
	cin >> tcs;
	while (tcs--) {
		solve();
	}
}