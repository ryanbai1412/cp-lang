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

const int maxn = 1e6 + 5;

struct Node {
	Node *l, *r;
} nodes[maxn];

void solve() {
	int n, q;
	cin >> n >> q;
	nodes[0].r = &nodes[1];
	for (int i = 1; i <= n; i++) {
		nodes[i].l = &nodes[i - 1];
		nodes[i].r = &nodes[i + 1];
	}
	while (q--) {
		int l, r, k;
		cin >> l >> r >> k;
		auto nk = nodes[k].r;
		auto nl = nodes[l].l;
		auto nr = nodes[r].r;
		{
			nodes[k].r = &nodes[l];
			nodes[l].l = &nodes[k];
			nodes[r].r = nk;
			nk->l = &nodes[r];
		}
		{
			nl->r = nr;
			nr->l = nl;
		}
	}
	Node* cur = &nodes[0];
	for (int i = 0; i < n; i++) {
		cur = cur->r;
		cout << cur - nodes << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}