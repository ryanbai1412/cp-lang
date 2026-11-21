#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>

using i64 = int64_t;

int main() {
    const i64 n = 500000;
    std::vector<i64> s(n);
    i64 x = 987654321;
    for (i64 i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        s[i] = (x >> 16) & 3;
    }
    // Manacher
    std::vector<i64> odd(n, 0), even(n, 0);
    i64 l = 0, r = -1;
    for (i64 i = 0; i < n; i++) {
        i64 k = i > r ? 1 : std::min(odd[l + r - i], r - i + 1);
        while (i - k >= 0 && i + k < n && s[i - k] == s[i + k]) k++;
        odd[i] = k;
        if (i + k - 1 > r) { l = i - k + 1; r = i + k - 1; }
    }
    l = 0; r = -1;
    for (i64 i = 0; i < n; i++) {
        i64 k = i > r ? 0 : std::min(even[l + r - i + 1], r - i + 1);
        while (i - k - 1 >= 0 && i + k < n && s[i - k - 1] == s[i + k]) k++;
        even[i] = k;
        if (i + k - 1 > r) { l = i - k; r = i + k - 1; }
    }
    i64 pal_cnt = 0, best_len = 0;
    for (i64 i = 0; i < n; i++) {
        pal_cnt += odd[i] + even[i];
        best_len = std::max(best_len, 2 * odd[i] - 1);
        best_len = std::max(best_len, 2 * even[i]);
    }
    // Duval Lyndon factorization
    std::vector<i64> lyn;
    i64 i = 0;
    while (i < n) {
        i64 j = i + 1, k = i;
        while (j < n && s[k] <= s[j]) {
            if (s[k] < s[j]) k = i;
            else k++;
            j++;
        }
        while (i <= k) {
            lyn.push_back(i);
            i += j - k;
        }
    }
    // Booth / Duval min rotation
    i64 a = 0, b = 1, k = 0;
    while (a < n && b < n && k < n) {
        i64 ca = s[(a + k) % n], cb = s[(b + k) % n];
        if (ca == cb) {
            k++;
        } else {
            if (ca > cb) a += k + 1;
            else b += k + 1;
            if (a == b) b++;
            k = 0;
        }
    }
    i64 rot = std::min(a, b);
    std::cout << (pal_cnt + best_len + (i64)lyn.size() + rot) << "\n";
}
