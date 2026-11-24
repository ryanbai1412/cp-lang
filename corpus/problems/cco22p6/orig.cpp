// CCO 2022 P6 Good Game: written reference. Only runs matter: a run of length >= 2
// ("big") can be removed whole, which merges its neighbours into a big run. A sequence of
// an odd number L = 2H + 1 of runs collapses to nothing iff its middle run is big, or the
// nearest big runs left and right of the middle sit at distances a and c with a + c <= H:
// collapsing c times at the left one brings the right one to the middle. An even number of
// runs must split into two such odd parts. Both rules are checked against an exhaustive
// search of the game.
#include <bits/stdc++.h>
using namespace std;

vector<int> r, prv, nxt;  // run lengths; nearest big run at or before / at or after i
string out;
long long K = 0;

bool odd_ok(int lo, int hi) {  // runs [lo, hi), odd count
    int L = hi - lo, H = (L - 1) / 2, mid = lo + H;
    if (r[mid] >= 2) return true;
    int p = prv[mid], q = nxt[mid];
    return p >= lo && q < hi && (mid - p) + (q - mid) <= H;
}

void emit_run(long long pos, int len) {
    char b[32];
    if (len & 1) { K++; out.append(b, snprintf(b, sizeof b, "%lld 3\n", pos)); len -= 3; }
    for (; len; len -= 2) { K++; out.append(b, snprintf(b, sizeof b, "%lld 2\n", pos)); }
}

// clear runs [lo, hi) which sit at the start of the board
void clear_part(int lo, int hi) {
    int L = hi - lo, H = (L - 1) / 2, mid = lo + H;
    vector<int> left(r.begin() + lo, r.begin() + mid);  // top = back
    deque<int> right(r.begin() + mid + 1, r.begin() + hi);
    long long sumLeft = 0;
    for (int x : left) sumLeft += x;
    int cur = r[mid];
    auto collapse = [&](int steps) {
        while (steps--) {
            emit_run(sumLeft + 1, cur);
            int a = left.back(); left.pop_back();
            sumLeft -= a;
            cur = a + right.front(); right.pop_front();
        }
    };
    if (r[mid] < 2) {
        int p = prv[mid], q = nxt[mid], a = mid - p, c = q - mid;
        // restart at the left big run p
        while ((int)left.size() > p - lo) right.push_front(cur), cur = left.back(), sumLeft -= cur, left.pop_back();
        collapse(c);
        left.push_back(cur), sumLeft += cur;
        for (int i = 0; i < a - 1; i++) left.push_back(right.front()), sumLeft += right.front(), right.pop_front();
        cur = right.front(), right.pop_front();
    }
    collapse((int)left.size());
    emit_run(1, cur);
}

int main() {
    int n;
    string s;
    cin >> n >> s;
    for (int i = 0; i < n;) {
        int j = i;
        while (j < n && s[j] == s[i]) j++;
        r.push_back(j - i), i = j;
    }
    int k = r.size();
    prv.assign(k, -1), nxt.assign(k, k);
    for (int i = 0, last = -1; i < k; i++) { if (r[i] >= 2) last = i; prv[i] = last; }
    for (int i = k - 1, last = k; i >= 0; i--) { if (r[i] >= 2) last = i; nxt[i] = last; }
    int split = -1;
    if (k % 2) {
        if (odd_ok(0, k)) split = k;
    } else {
        for (int i = 1; i < k; i += 2)
            if (odd_ok(0, i) && odd_ok(i, k)) { split = i; break; }
    }
    if (split < 0) { puts("-1"); return 0; }
    clear_part(0, split);
    if (split < k) clear_part(split, k);
    printf("%lld\n", K);
    fwrite(out.data(), 1, out.size(), stdout);
}
