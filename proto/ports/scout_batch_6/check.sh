#!/bin/bash
here=$(cd "$(dirname "$0")" && pwd)
exec bash "$here/today/stress.sh" "${1:-50}" "${2:-6}"
