import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ci_unstick import verdict  # noqa: E402

T0 = 1_700_000_000


def stamp(t):
    from datetime import datetime, timezone
    return datetime.fromtimestamp(T0 + t, timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


RUN = {"run_started_at": stamp(0)}


def done(end):
    return {"status": "completed", "completed_at": stamp(end), "steps": []}


def complete_job_waits(end, last="completed"):
    return {"status": "in_progress", "completed_at": None, "steps": [
        {"name": "Fast gate", "status": last, "completed_at": stamp(end) if last == "completed" else None},
        {"name": "Complete job", "status": "pending", "completed_at": None}]}


class VerdictTest(unittest.TestCase):
    def test_a_job_left_in_complete_job_hangs_the_run_after_three_minutes(self):
        jobs = [done(100), complete_job_waits(370)]
        self.assertIsNone(verdict(RUN, jobs, T0 + 370 + 179))
        self.assertEqual(verdict(RUN, jobs, T0 + 370 + 200), ("hung", 200))

    def test_a_job_still_running_a_step_is_working(self):
        self.assertIsNone(verdict(RUN, [done(100), complete_job_waits(0, last="in_progress")], T0 + 600))

    def test_a_queued_job_is_working(self):
        queued = {"status": "queued", "completed_at": None, "steps": []}
        self.assertIsNone(verdict(RUN, [done(100), queued], T0 + 600))

    def test_a_long_run_still_working_is_only_reported(self):
        working = complete_job_waits(0, last="in_progress")
        self.assertEqual(verdict(RUN, [working], T0 + 16 * 60), ("long", 16 * 60))


if __name__ == "__main__":
    unittest.main()
