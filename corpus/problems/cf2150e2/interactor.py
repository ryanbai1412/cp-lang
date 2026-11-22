#!/usr/bin/env python3
import importlib.util, os, sys
_here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(_here, "..", "..", "lib"))
from interactor_lib import run_interactor

_spec = importlib.util.spec_from_file_location(
    "cf2150e1_interactor", os.path.join(_here, "..", "cf2150e1", "interactor.py")
)
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)

if __name__ == "__main__":
    run_interactor(_mod.drive)
