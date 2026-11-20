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
		a--;
	}
	int perm[n];
	iota(perm, perm + n, 0);
	int poss[n] {};
	do {
		for (int i = 0; i < n; i++) {
			int prv = i ? i - 1 : n - 1;
			int nxt = i + 1 < n ? i + 1 : 0;
			if (min(perm[prv], perm[nxt]) != arr[i]) {
				goto loop;
			}
		}
		for (int i = 0; i < n; i++) {
			poss[i] |= 1 << perm[i];
		}
	loop:;
	} while (next_permutation(perm, perm + n));
	for (int i = 0; i < n; i++) {
		if (__builtin_popcount(poss[i]) == 1) {
			cout << __builtin_ctz(poss[i]) + 1;
		} else {
			cout << 0;
		}
		cout << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int tcs;
	cin >> tcs;
	while (tcs--) {
		solve();
	}
}