#include "bits/extc++.h"

using namespace std;

template <class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template <class T, class U = less<T>>
using rt = __gnu_pbds::tree<T,
							__gnu_pbds::null_type,
							U,
							__gnu_pbds::rb_tree_tag,
							__gnu_pbds::tree_order_statistics_node_update>;

template <class T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <class T, class... U>
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

// imagine a language where int = long
#define long int64_t

// typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

namespace treap {

mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());

struct Node {
	char c;
	int sz, pri;
	Node *l, *r;

	static Node* alloc() {
		Node* heaps[100];
		static int aind = -1, csz = 16, cind = 16;

		if (csz == cind) {
			csz <<= 1;
			aind++;
			heaps[aind] = new Node[csz];
			cind = 0;
		}

		return &heaps[aind][cind++];
	}

	static Node* falloc(char c) {
		Node* n = alloc();
		*n = from(c);
		return n;
	}

	static Node from(char c) { return Node{c, 1, cowng(), nullptr, nullptr}; }
};

using Treap = Node*;

int size(Treap t) {
	return t ? t->sz : 0;
}

void maintain(Treap t) {
	if (t) {
		t->sz = 1 + size(t->l) + size(t->r);
	}
}

void split(Treap t, int ind, Treap& l, Treap& r) {
	if (!t) {
		l = r = nullptr;
	} else {
		if (ind <= size(t->l)) {
			split(t->l, ind, l, t->l);
			r = t;
		} else {
			split(t->r, ind - size(t->l) - 1, t->r, r);
			l = t;
		}
	}
	maintain(t);
}

pair<Treap, Treap> split(Treap t, int ind) {
	Treap l{}, r{};
	split(t, ind, l, r);
	return {l, r};
}

void merge(Treap& t, Treap l, Treap r) {
	if (!(l && r)) {
		t = l ? l : r;
	} else {
		if (l->pri < r->pri) {
			merge(l->r, l->r, r);
			t = l;
		} else {
			merge(r->l, l, r->l);
			t = r;
		}
	}
	maintain(t);
}

Treap merge(Treap l, Treap r) {
	Treap t;
	merge(t, l, r);
	return t;
}

}  // namespace treap

using treap::Treap, treap::Node;

ostream& operator<<(ostream& out, Treap t) {
	if (t) {
		out << t->l;
		out << t->c;
		out << t->r;
	}
	return out;
}

void solve() {
	const int buf_len = 3e6;
	static char buf[buf_len];
	int q;
	cin >> q;
	auto rstring = [&](int len) -> string {
		string cur;
		while (sz(cur) < len) {
			cin.getline(buf, buf_len);
			cur += string(buf);
		};
		return cur;
	};
	Treap t{};
	int ptr = 0;
	while (q--) {
		dbg(t);
		cerr << endl;
		string type;
		cin >> type;
		if (type == "Move") {
			cin >> ptr;
		} else if (type == "Insert") {
			int n;
			cin >> n;
			string s = rstring(n);
			auto [l, r] = treap::split(t, ptr);
			for (auto& a : s) {
				l = treap::merge(l, Node::falloc(a));
			}
			t = treap::merge(l, r);
		} else if (type == "Delete") {
			int n;
			cin >> n;
			auto [l, r] = treap::split(t, ptr);
			dbg(l, r);
			auto [mid, ri] = treap::split(r, n);
			dbg(l, mid, ri);
			t = treap::merge(l, ri);
		} else if (type == "Get") {
			int n;
			cin >> n;
			auto [l, r] = treap::split(t, ptr);
			auto [mid, ri] = treap::split(r, n);
			cout << mid << endl;
			t = treap::merge(l, treap::merge(mid, ri));
		} else if (type == "Prev") {
			ptr--;
		} else if (type == "Next") {
			ptr++;
		} else {
			assert(false);
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}