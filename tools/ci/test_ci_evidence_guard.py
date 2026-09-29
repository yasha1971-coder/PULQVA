import unittest
from ci_evidence_guard import evaluate

SHA = '1' * 40
EXPECTED = ['linux', 'windows']
def job(name, status='completed', conclusion='success', sha=SHA):
    return dict(name=name, status=status, conclusion=conclusion, source_sha=sha)

class EvidenceGuardTests(unittest.TestCase):
    def test_all_expected_success(self):
        self.assertEqual(evaluate(SHA, EXPECTED, [job('linux'),job('windows')]).state,'PASS')
    def test_failed_child_while_sibling_running_is_not_green(self):
        v=evaluate(SHA, EXPECTED,[job('linux',conclusion='failure'),job('windows','in_progress',None)])
        self.assertEqual(v.state,'FAIL'); self.assertEqual(v.failed,('linux',))
    def test_pending_is_not_success(self):
        self.assertEqual(evaluate(SHA,EXPECTED,[job('linux'),job('windows','queued',None)]).state,'PENDING')
    def test_old_head_cannot_supply_missing_windows(self):
        v=evaluate(SHA,EXPECTED,[job('linux'),job('windows',sha='2'*40)])
        self.assertEqual(v.state,'INCOMPLETE');self.assertEqual(v.missing,('windows',))
    def test_missing_jobs_are_not_green(self):
        self.assertEqual(evaluate(SHA,EXPECTED,[job('linux')]).state,'INCOMPLETE')
    def test_no_jobs_are_not_green(self):
        self.assertEqual(evaluate(SHA,EXPECTED,[]).state,'INCOMPLETE')
    def test_skipped_neutral_cancelled_unknown_are_not_success(self):
        for outcome in ('skipped','neutral','cancelled',None,'unknown'):
            with self.subTest(outcome=outcome):
                self.assertEqual(evaluate(SHA,EXPECTED,[job('linux'),job('windows',conclusion=outcome)]).state,'INCOMPLETE')
    def test_attempts_must_be_disambiguated(self):
        self.assertEqual(evaluate(SHA,EXPECTED,[job('linux'),job('linux'),job('windows')]).state,'INCOMPLETE')
    def test_failure_is_not_hidden_by_missing_job(self):
        self.assertEqual(evaluate(SHA,EXPECTED,[job('linux',conclusion='failure')]).state,'FAIL')
    def test_expected_set_cannot_be_empty_or_duplicated(self):
        for expected in ([], ['linux','linux']):
            with self.assertRaises(ValueError):evaluate(SHA,expected,[])
    def test_sha_must_be_exact(self):
        with self.assertRaises(ValueError):evaluate('1680dc8',EXPECTED,[])
    def test_invalid_status_is_not_green(self):
        self.assertEqual(evaluate(SHA,EXPECTED,[job('linux'),job('windows','mystery',None)]).state,'INCOMPLETE')

if __name__=='__main__':unittest.main()
