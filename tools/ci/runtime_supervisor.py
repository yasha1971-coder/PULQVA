"""Bounded runtime orchestration using the existing matrix codec.

Backend owns an OS job/process group, not a cached PID list. This module does
not provide an OS launcher. Native containment admission is a separate gate.
"""
from matrix_receipt import AdmissionWindow, PersistedMatrix, require

class RuntimeSupervisor:
    def __init__(self, plan, directory, backend, clock, session_deadline):
        require([p['id'] for p in plan['probes']] ==
                ['GUI', 'SUBMIT', 'ROUTE', 'RESULTS', 'CLEANUP'],
                'unexpected runtime probe inventory')
        self.plan, self.directory = plan, directory
        self.backend, self.clock = backend, clock
        self.window = AdmissionWindow(session_deadline, clock)

    def run(self):
        # Backend preparation must not create a process. Failure precedes launch.
        self.backend.prepare()
        boundary = PersistedMatrix(self.plan, self.directory)
        failure = None
        with boundary as rec:
            try:
                start = self.clock()
                # Reserve complete window BEFORE creating any process too.
                require(self.window.deadline-start >= 120,
                        'insufficient time before launch')
                self.backend.launch_owned()
                gui = self.backend.observe_gui()
                rec.record('GUI', result='PASS' if gui else 'FAIL',
                           duration_ms=(self.clock()-start)*1000,
                           observed={'visible':gui}, evidence_refs=['backend:gui'])
                if gui:
                    start=self.clock()
                    # Backend may only submit after admission, exactly once.
                    self.window.acknowledge_submission()
                    ack = self.backend.submit_once()
                    require(ack is True, 'query submission not acknowledged')
                    # Submission latency cannot silently consume readiness time.
                    require(self.window.deadline-self.clock() >= 120,
                            'insufficient time after submission acknowledgement')
                    self.window.submitted_at=self.clock()
                    rec.record('SUBMIT',result='PASS',
                               duration_ms=(self.clock()-start)*1000,
                               observed={'acknowledged':True}, evidence_refs=['backend:submit'])
                    start=self.clock()
                    route=self.backend.observe_route(
                        self.window.submitted_at+self.window.readiness)
                    status=self.window.outcome(route_ready=route is True)
                    rec.record('ROUTE',result='PASS' if status=='READY' else 'FAIL',
                               duration_ms=(self.clock()-start)*1000,
                               observed={'classification':status}, evidence_refs=['backend:route'])
                    # No results operation on missing readiness or expired budget.
                    if status=='READY':
                        start=self.clock()
                        count=self.backend.observe_results()
                        require(isinstance(count,int) and not isinstance(count,bool)
                                and count>=0,'invalid result count')
                        require(self.clock()<self.window.deadline-self.window.cleanup,
                                'result observation exceeded budget')
                        rec.record('RESULTS',result='PASS' if count>=2 else 'FAIL',
                                   duration_ms=(self.clock()-start)*1000,
                                   observed={'real_result_count':count},
                                   evidence_refs=['backend:results'])
            except Exception as exc:
                failure=type(exc).__name__  # no raw input/path/error disclosure
            finally:
                start=self.clock()
                cleanup_error=False
                try:
                    self.backend.request_close()
                    # Query current job membership: catches restarted descendants.
                    if self.backend.members():
                        self.backend.terminate_owned_job()
                    clean = not self.backend.members() and not self.backend.owned_ports()
                except Exception:
                    cleanup_error=True
                    clean=False
                # Preserve order and all missing probes even on exceptions.
                for spec in self.plan['probes'][:-1]:
                    if spec['id'] not in rec._rows:
                        rec.record(spec['id'],result='SKIP',duration_ms=None,
                                   observed=None,evidence_refs=[],
                                   reason='preceding_probe_unavailable',
                                   timing_reason='probe_not_started')
                rec.record('CLEANUP',result='PASS' if clean else 'FAIL',
                           duration_ms=(self.clock()-start)*1000,
                           observed={'empty_owned_job_and_ports':clean,
                                     'cleanup_error':cleanup_error,
                                     'collector_error_class':failure},
                           evidence_refs=['backend:cleanup'])
        return boundary.receipt

