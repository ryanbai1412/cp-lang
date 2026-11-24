// Reference for NOI WC 2017 P2 (no accepted submission on DMOJ; written for the corpus), inside
// the statement's template (next_integer, output_arr and the three readers are verbatim).
// Sorting: an in-place most-significant-byte bucket pass, then a 24-bit LSD radix per bucket.
// Game: per query, popcounts over 64-bit words of the three "first row wins" masks, reading
// both rows word-aligned from 64 pre-shifted copies of each row's three strategy bitsets.
// Parentheses: the O(n^2) balance DP mod 2^32, keeping only balances of the step's parity and
// at most the remaining length, as one in-place a[p] += a[p + 1] sweep per '?'.
#include <stdio.h>
#include <string.h>
#include <algorithm>
#include <vector>

typedef unsigned int u32;
typedef unsigned long long u64;

inline u32 next_integer(u32 x) {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return x;
}

bool output_arr(void *a, u32 size) {
    if (size % 4) {
        return puts("-1"), 0;
    }

    u32 blocks = size / 4;
    u32 *A = (u32 *)a;
    u32 ret = size;
    u32 x = 23333333;
    for (u32 i = 0; i < blocks; i++) {
        ret = ret ^ (A[i] + x);
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
    }

    return printf("%u\n", ret), 1;
}

void sort(u32 *a, int n) {
    size_t cnt[257] = {0}, head[256], tail[256];
    for (int i = 0; i < n; i++) cnt[(a[i] >> 24) + 1]++;
    for (int b = 0; b < 256; b++) cnt[b + 1] += cnt[b];
    for (int b = 0; b < 256; b++) head[b] = cnt[b], tail[b] = cnt[b + 1];
    for (int b = 0; b < 256; b++)
        while (head[b] < tail[b]) {
            u32 v = a[head[b]];
            int d = v >> 24;
            if (d == b) { head[b]++; continue; }
            while (d != b) {
                u32 w = a[head[d]];
                a[head[d]++] = v;
                v = w, d = v >> 24;
            }
            a[head[b]++] = v;
        }
    size_t big = 0;
    for (int b = 0; b < 256; b++) big = std::max(big, cnt[b + 1] - cnt[b]);
    std::vector<u32> tmp(big);
    for (int b = 0; b < 256; b++) {
        u32 *p = a + cnt[b], *q = tmp.data();
        size_t m = cnt[b + 1] - cnt[b];
        for (int sh = 0; sh < 24; sh += 8) {
            size_t c[256] = {0};
            for (size_t i = 0; i < m; i++) c[(p[i] >> sh) & 255]++;
            size_t s = 0;
            for (int d = 0; d < 256; d++) { size_t t = c[d]; c[d] = s; s += t; }
            for (size_t i = 0; i < m; i++) q[c[(p[i] >> sh) & 255]++] = p[i];
            std::swap(p, q);
        }
        if (p != a + cnt[b]) memcpy(a + cnt[b], p, m * sizeof(u32));
    }
}

// ===== header ======

namespace Sorting {
void init_data(u32 *a, int n, u32 seed) {
    for (int i = 0; i < n; i++) {
        seed = next_integer(seed);
        a[i] = seed;
    }
}

void main() {
    int n;
    u32 seed;
    scanf("%d%u", &n, &seed);

    u32 *a = new u32[n];
    init_data(a, n, seed);

    sort(a, n);

    output_arr(a, n * sizeof(u32));
}
}

void solve(int n, int q, char *s1, char *s2, int *q_x, int *q_y, int *q_len, u32 *anss) {
    // sh[r][c][k]: row r's bitset of strategy c, shifted right by k bits.
    int W = (n + 63) / 64 + 2;
    std::vector<u64> sh[2][3];
    for (int r = 0; r < 2; r++)
        for (int c = 0; c < 3; c++) {
            const char *s = r ? s2 : s1;
            std::vector<u64> base(W + 1, 0);
            for (int i = 0; i < n; i++)
                if (s[i] - '0' == c) base[i / 64] |= 1ULL << (i % 64);
            sh[r][c].assign((size_t)64 * W, 0);
            for (int k = 0; k < 64; k++)
                for (int w = 0; w < W; w++)
                    sh[r][c][(size_t)k * W + w] = k ? (base[w] >> k) | (base[w + 1] << (64 - k)) : base[w];
        }
    for (int t = 0; t < q; t++) {
        int x = q_x[t], y = q_y[t], l = q_len[t];
        // 0 (rock) beats 1 (scissors), 1 beats 2 (paper), 2 beats 0.
        const u64 *a[3], *b[3];
        for (int c = 0; c < 3; c++)
            a[c] = sh[0][c].data() + (size_t)(x % 64) * W + x / 64,
            b[c] = sh[1][c].data() + (size_t)(y % 64) * W + y / 64;
        u32 res = 0;
        int full = l / 64;
        for (int w = 0; w < full; w++)
            res += __builtin_popcountll((a[0][w] & b[1][w]) | (a[1][w] & b[2][w]) | (a[2][w] & b[0][w]));
        if (l % 64) {
            u64 mask = (1ULL << (l % 64)) - 1;
            res += __builtin_popcountll(((a[0][full] & b[1][full]) | (a[1][full] & b[2][full]) | (a[2][full] & b[0][full])) & mask);
        }
        anss[t] = res;
    }
}

namespace Game {
void main() {
    int n, q;
    scanf("%d%d", &n, &q);

    char *s1 = new char[n + 1];
    char *s2 = new char[n + 1];
    scanf("%s%s", s1, s2);

    u32 *anss = new u32[q];
    int *q_x = new int[q];
    int *q_y = new int[q];
    int *q_len = new int[q];

    for (int i = 0; i < q; i++) {
        scanf("%d%d%d", q_x + i, q_y + i, q_len + i);
    }

    solve(n, q, s1, s2, q_x, q_y, q_len, anss);

    output_arr(anss, q * sizeof(u32));
}
}

u32 solve(int n, char *s) {
    if (n % 2) return 0;
    // f[base + j] is the count for balance 2j + (i & 1) after i characters; f[base - 1] stays 0.
    std::vector<u32> f(2 * (size_t)n + 8, 0);
    size_t base = n + 2;
    f[base] = 1;
    for (int i = 0; i < n; i++) {
        int odd = i & 1;
        // Balances needed after this step: at most min(i + 1, n - i - 1), plus one of slack
        // so that stale values above the bound never flow back into it.
        int lim = std::min(i + 1, n - i - 1) + 2;
        size_t J = lim / 2 + 1;
        char c = s[i];
        if (c == '(') {
            // even: b = 2j -> 2j + 1 keeps j; odd: b = 2j + 1 -> 2j + 2 is j + 1.
            if (odd) base--;
        } else if (c == ')') {
            // even: 2j -> 2j - 1 is j - 1 (balance 0 dies); odd: 2j + 1 -> 2j keeps j.
            if (!odd) f[base] = 0, base++;
        } else {
            // even: new[j] = old[j] + old[j + 1] in place; odd: new[j] = old[j - 1] + old[j],
            // the same sweep one slot to the left (old[-1] = f[base - 1] = 0).
            if (odd) base--;
            u32 *p = f.data() + base;
            for (size_t j = 0; j <= J; j++) p[j] += p[j + 1];
        }
    }
    return f[base];
}

namespace Parentheses {
void main() {
    int n;
    scanf("%d", &n);

    char *s = new char[n + 1];
    scanf("%s", s);

    u32 ans;
    ans = solve(n, s);

    printf("%u\n", ans);
}
}

int main() {
    int task_id;
    scanf("%d", &task_id);

    switch (task_id) {
        case 1:
            Sorting::main();
            break;
        case 2:
            Game::main();
            break;
        case 3:
            Parentheses::main();
            break;
    }

    return 0;
}
