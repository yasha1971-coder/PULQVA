import copy
import json
from pathlib import Path
import tempfile
import unittest
from test_matrix_receipt import fixture_plan
from runtime_supervisor import RuntimeSupervisor

class FakeBackend:
    """Synthetic job identity, never a native process/PID killer."""
    def __init__(self):
        self.now=0;self.events=[];self.job=[];self.ports=[];self.gui=True
        self.ready=True;self.count=2;self.submit_delay=0;self.query_count=0
        self.cleanup_fails=False;self.launch_fails=False
    def prepare(self):self.events.append('prepare')
    def launch_owned(self):
        if self.launch_fails:raise OSError('synthetic')
        self.job=[('root',1),('arti',2)];self.events.append('launch')
    def observe_gui(self):return self.gui
    def submit_once(self):
        self.query_count+=1;self.now+=self.submit_delay;return True
    def observe_route(self,deadline):
        self.job=[('root',1),('arti-restarted',3)]
        if not self.ready:self.now=deadline
        return self.ready
    def observe_results(self):self.events.append('results');return self.count
    def request_close(self):self.events.append('close')
    def members(self):return list(self.job)
    def terminate_owned_job(self):
        self.events.append('terminate-owned')
        if not self.cleanup_fails:self.job=[]
    def owned_ports(self):return self.ports

class SupervisorTests(unittest.TestCase):
    def setUp(self):
        self.t=tempfile.TemporaryDirectory();self.addCleanup(self.t.cleanup)
        self.root=Path(self.t.name);self.b=FakeBackend()
        p=fixture_plan();spec=p['probes'][0]
        p['probes']=[dict(copy.deepcopy(spec),id=x) for x in
                     ['GUI','SUBMIT','ROUTE','RESULTS','CLEANUP']]
        self.s=RuntimeSupervisor(p,self.root,self.b,lambda:self.b.now,180)
    def test_complete_receipt(self):
        r=self.s.run();self.assertTrue(r['accepted'])
        self.assertEqual(self.b.query_count,1)
        self.assertTrue((self.root/'matrix_receipt.json').exists())
    def test_early_launch_error_cleanup_and_complete_inventory(self):
        self.b.launch_fails=True;r=self.s.run()
        self.assertFalse(r['accepted']);self.assertEqual(len(r['probes']),5)
        self.assertIn('close',self.b.events)
    def test_route_failure_suppresses_results_only(self):
        self.b.ready=False;r=self.s.run()
        self.assertNotIn('results',self.b.events);self.assertFalse(r['accepted'])
        self.assertEqual(r['probes'][2]['observed']['classification'],'READINESS_WINDOW_EXPIRED')
        self.assertEqual(r['probes'][-1]['result'],'PASS')
    def test_slow_submit_no_route_acceptance(self):
        self.b.submit_delay=61;r=self.s.run()
        self.assertFalse(r['accepted']);self.assertEqual(self.b.query_count,1)
        self.assertNotIn('results',self.b.events)
    def test_insufficient_initial_time_no_launch(self):
        self.b.now=61;r=self.s.run()
        self.assertNotIn('launch',self.b.events);self.assertFalse(r['accepted'])
    def test_restarted_child_in_current_job_is_cleaned(self):
        r=self.s.run();self.assertEqual(self.b.job,[])
        self.assertIn('terminate-owned',self.b.events)
    def test_cleanup_failure_blocks_pass(self):
        self.b.cleanup_fails=True;r=self.s.run()
        self.assertFalse(r['accepted']);self.assertEqual(r['probes'][-1]['result'],'FAIL')
    def test_port_leftover_blocks_pass(self):
        self.b.ports=[9150];r=self.s.run();self.assertFalse(r['accepted'])
    def test_gui_failure_no_query_but_cleanup(self):
        self.b.gui=False;r=self.s.run();self.assertEqual(self.b.query_count,0)
        self.assertFalse(r['accepted']);self.assertIn('close',self.b.events)
    def test_one_result_fails(self):
        self.b.count=1;r=self.s.run();self.assertFalse(r['accepted'])

if __name__=='__main__':unittest.main()

