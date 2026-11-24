# shellcheck shell=bash
# The one retry in CI and scripts (handbook/guidelines/ci-checklist.md
# ci/R34): a network fetch, or a push racing main, tried at most
# $RETRY_TRIES times (default 3), each failed try named, the last one failing.
# debt-ratchet's ci_retries counts every retry outside this file.
# Source it, then `retry cmd...` or `fetch URL [curl args]`.
retry() {
    local tries=${RETRY_TRIES:-3} n
    for ((n = 1; ; n++)); do
        "$@" && return 0
        if [ "$n" -ge "$tries" ]; then
            echo "retry: gave up after $tries tries: $*" >&2
            return 1
        fi
        echo "retry: try $n of $tries failed: $*" >&2
        sleep "${RETRY_WAIT:-5}"
    done
}
fetch() { retry curl -fsSL "$@"; }
