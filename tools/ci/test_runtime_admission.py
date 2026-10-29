"""Offline admission tests: no app, Tor, subprocess or network launch."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from matrix_receipt import AdmissionWindow, PersistedMatrix, verify
from test_matrix_receipt import fixture_plan, add

class BudgetTests(unittest.TestCase):
    def gate(self, now=0):
        self.now = now
        return AdmissionWindow(180, lambda: self.now)

    def test_full_window(self):
        g=self.gate(60)
        self.assertEqual(g.acknowledge_submission(),150)
        self.now=149
        self.assertEqual(g.outcome(),'PENDING')
        self.now=150
        self.assertEqual(g.outcome(),'READINESS_WINDOW_EXPIRED')

    def test_late_submission_rejected(self):
        g=self.gate(60.001)
        with self.assertRaises(ValueError): g.acknowledge_submission()
        self.assertIsNone(g.submitted_at)

    def test_no_acknowledgement(self):
        with self.assertRaises(ValueError): self.gate().outcome()

    def test_duplicate_query(self):
        g=self.gate();g.acknowledge_submission()
        with self.assertRaises(ValueError): g.acknowledge_submission()

    def test_cleanup_reserve_is_harness_interruption(self):
        g=self.gate();g.acknowledge_submission();self.now=160
        self.assertEqual(g.outcome(),'HARNESS_INTERRUPTED')
        self.assertEqual(g.outcome(route_ready=True),'HARNESS_INTERRUPTED')

    def test_clock_reversal(self):
        g=self.gate(10);g.acknowledge_submission();self.now=9
        with self.assertRaises(ValueError):g.outcome()

    def test_invalid_budget_and_clock(self):
        for v in (float('nan'),float('inf'),-1,True):
            with self.assertRaises(ValueError):AdmissionWindow(v,lambda:0)
        g=self.gate();self.now=float('nan')
        with self.assertRaises(ValueError):g.acknowledge_submission()

    def test_ready_only_when_observed(self):
        g=self.gate();g.acknowledge_submission()
        self.assertEqual(g.outcome(),'PENDING')
        self.assertEqual(g.outcome(route_ready=True),'READY')

class PersistenceTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)
        self.plan=fixture_plan()
        cleanup=copy.deepcopy(self.plan['probes'][-1]);cleanup['id']='CLEANUP'
        self.plan['probes'].append(cleanup)

    def receipt(self):
        return json.loads((self.root/'matrix_receipt.json').read_text())

    def test_complete(self):
        with PersistedMatrix(self.plan,self.root) as r:
            for p in self.plan['probes']:add(r,p['id'])
        self.assertEqual(verify(self.plan,self.receipt()),'PASS')

    def test_early_exit_retains_every_probe(self):
        with PersistedMatrix(self.plan,self.root):pass
        receipt=self.receipt()
        self.assertEqual(len(receipt['probes']),3)
        self.assertFalse(receipt['accepted'])
        self.assertTrue(all(x['result']=='SKIP' for x in receipt['probes']))

    def test_exception_persists_and_propagates(self):
        with self.assertRaises(RuntimeError):
            with PersistedMatrix(self.plan,self.root):raise RuntimeError('fixture')
        self.assertFalse(self.receipt()['accepted'])

    def test_missing_cleanup_cannot_pass(self):
        with PersistedMatrix(self.plan,self.root) as r:
            add(r,'A');add(r,'B')
        self.assertFalse(self.receipt()['accepted'])

    def test_failed_cleanup_cannot_pass(self):
        with PersistedMatrix(self.plan,self.root) as r:
            add(r,'A');add(r,'B');add(r,'CLEANUP','FAIL')
        self.assertEqual(verify(self.plan,self.receipt()),'FAIL')

    def test_no_overwrite_previous_manifest(self):
        with PersistedMatrix(self.plan,self.root):pass
        original=(self.root/'matrix_receipt.json').read_bytes()
        with self.assertRaises(FileExistsError):
            with PersistedMatrix(self.plan,self.root):pass
        self.assertEqual(original,(self.root/'matrix_receipt.json').read_bytes())

    def test_duplicate_probe_rejected_receipt_still_saved(self):
        with self.assertRaises(ValueError):
            with PersistedMatrix(self.plan,self.root) as r:add(r,'A');add(r,'A')
        self.assertFalse(self.receipt()['accepted'])

    def test_wrong_provenance_rejected(self):
        with PersistedMatrix(self.plan,self.root) as r:
            for p in self.plan['probes']:add(r,p['id'])
        receipt=self.receipt();receipt['identity']['source_sha']='f'*40
        with self.assertRaises(ValueError):verify(self.plan,receipt)

    def test_cleanup_required_in_plan(self):
        with self.assertRaises(ValueError):PersistedMatrix(fixture_plan(),self.root)

if __name__=='__main__':unittest.main()

