# shellcheck shell=bash
# The one retry in CI and scripts (handbook/guidelines/ci-checklist.md
# ci/R34): a network fetch, or a push racing main, tried at most
# $RETRY_TRIES times (default 3), each failed try named, the last one failing.
# debt-ratchet's ci_retries counts every retry outside this file.
# Source it, then `retry cmd...` or `fetch URL [curl args]`.
retry() {
    # Prefixed locals: the retried command runs in this scope and may set its own.
    local _retry_tries=${RETRY_TRIES:-3} _retry_n
    for ((_retry_n = 1; ; _retry_n++)); do
        "$@" && return 0
        if [ "$_retry_n" -ge "$_retry_tries" ]; then
            echo "retry: gave up after $_retry_tries tries: $*" >&2
            return 1
        fi
        echo "retry: try $_retry_n of $_retry_tries failed: $*" >&2
        sleep "${RETRY_WAIT:-5}"
    done
}
fetch() { retry curl -fsSL "$@"; }
