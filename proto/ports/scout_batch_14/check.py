#!/usr/bin/env python3
"""Entry point for bench/corpus_registry.py; delegates to today/stress.py."""
import os, subprocess, sys
here = os.path.dirname(os.path.abspath(__file__))
sys.exit(subprocess.call([sys.executable, os.path.join(here, "today", "stress.py")] + sys.argv[1:]))
