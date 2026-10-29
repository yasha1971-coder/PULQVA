"""Real harmless Windows process fixtures, no PULQVA/Tor or external network."""
import os
import ctypes as C
from ctypes import wintypes as W
from pathlib import Path
import socket
import sys
import tempfile
import time
import unittest
from runtime_windows_job import WindowsJob

@unittest.skipUnless(os.name=='nt','Windows native job API')
class WindowsJobTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)

    def wait_for(self, fn, timeout=4):
        end=time.monotonic()+timeout
        while time.monotonic()<end:
            if fn():return
            time.sleep(.02)
        self.fail('fixture deadline')

    def launch(self,job,source):
        return job.launch([sys.executable,'-B','-c',source],self.root)

    def test_owned_root_cleanup(self):
        with WindowsJob() as j:
            pid=self.launch(j,'import time;time.sleep(20)')
            self.assertIn(pid,j.members())
            self.assertTrue(j.cleanup());self.assertEqual(j.members(),())

    def test_descendant_and_loopback_port_cleanup(self):
        marker=self.root/'ready'
        child=("import socket,time,pathlib;s=socket.socket();s.bind(('127.0.0.1',0));"
               "s.listen();pathlib.Path("+repr(str(marker))+").write_text(str(s.getsockname()[1]));time.sleep(20)")
        root="import subprocess,sys,time;subprocess.Popen([sys.executable,'-B','-c',"+repr(child)+"]);time.sleep(20)"
        with WindowsJob() as j:
            self.launch(j,root)
            self.wait_for(lambda:marker.exists() and marker.stat().st_size>0)
            port=int(marker.read_text());self.assertGreaterEqual(len(j.members()),2)
            j.cleanup();self.assertEqual(j.members(),())
            def port_released():
                with socket.socket() as probe:
                    probe.setsockopt(socket.SOL_SOCKET,socket.SO_EXCLUSIVEADDRUSE,1)
                    try:probe.bind(('127.0.0.1',port));return True
                    except OSError as exc:
                        if exc.winerror != 10048:raise
                        return False
            self.wait_for(port_released)
            self.assertEqual(j.members(),())

    def test_wall_deadline_cleans_without_model_action(self):
        with WindowsJob() as j:
            self.launch(j,'import time;time.sleep(20)')
            j.arm_deadline(time.monotonic()+.15)
            self.wait_for(lambda:not j.members())

    def test_unrelated_job_untouched(self):
        with WindowsJob() as other,WindowsJob() as tested:
            pid=self.launch(other,'import time;time.sleep(20)')
            self.launch(tested,'import time;time.sleep(20)')
            tested.cleanup()
            self.assertIn(pid,other.members())

    def test_invalid_executable_no_process(self):
        with WindowsJob() as j:
            with self.assertRaises(OSError):j.launch([str(self.root/'missing.exe')],self.root)
            self.assertEqual(j.members(),())

    def test_kill_on_handle_close(self):
        j=WindowsJob()
        process=None
        k=j.k
        k.OpenProcess.argtypes=[W.DWORD,W.BOOL,W.DWORD];k.OpenProcess.restype=W.HANDLE
        k.WaitForSingleObject.argtypes=[W.HANDLE,W.DWORD];k.WaitForSingleObject.restype=W.DWORD
        try:
            pid=self.launch(j,'import time;time.sleep(20)')
            process=k.OpenProcess(0x100000,False,pid)
            self.assertTrue(process)
            j.close()
            self.assertEqual(k.WaitForSingleObject(process,4000),0)
        finally:
            j.close()
            if process:k.CloseHandle(process)
        with self.assertRaises(RuntimeError):j.members()

if __name__=='__main__':unittest.main()

