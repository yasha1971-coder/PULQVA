"""Linux ownership helper for the existing RuntimeSupervisor backend.
Run in a dedicated owner process. Subreaper adoption keeps detached/restarted
descendants owned; pidfds prevent signalling a recycled numeric PID.
The owner must be an agent-independent deadline guardian, not the GUI process.
This is lifecycle ownership, not an egress sandbox or delegated cgroup.
"""
import ctypes, os, signal, subprocess, time

def _children(pid):
    try:
        return [int(x) for x in open(f"/proc/{pid}/task/{pid}/children").read().split()]
    except (FileNotFoundError, ProcessLookupError):
        return []

def _identity(pid):
    try:
        raw=open(f"/proc/{pid}/stat").read()
        fields=raw[raw.rfind(")")+2:].split()
        return (pid,int(fields[19]))
    except (FileNotFoundError,ProcessLookupError):
        return None

class LinuxOwnedJob:
    def __init__(self):
        if not hasattr(os,"pidfd_open") or not hasattr(signal,"pidfd_send_signal"):
            raise RuntimeError("pidfd required")
        if _children(os.getpid()):
            raise RuntimeError("requires dedicated owner with no pre-existing children")
        libc=ctypes.CDLL(None,use_errno=True)
        if libc.prctl(36,1,0,0,0)!=0: # PR_SET_CHILD_SUBREAPER
            raise OSError(ctypes.get_errno(),"subreaper unavailable")
        self.owner=os.getpid()
        self.process=None
    def members(self):
        todo=_children(self.owner); seen=set(); members=[]
        while todo:
            pid=todo.pop()
            if pid in seen:continue
            seen.add(pid); todo.extend(_children(pid))
            identity=_identity(pid)
            if identity:members.append(identity)
        return members
    def launch(self,argv,**kwargs):
        if self.process is not None:raise RuntimeError("single launch")
        self.process=subprocess.Popen(argv,start_new_session=True,**kwargs)
        return self.process
    def _signal(self,sig):
        # Refresh actual ownership, open pidfd, then recheck ownership/identity.
        for identity in self.members():
            try:
                fd=os.pidfd_open(identity[0],0)
                try:
                    if _identity(identity[0])==identity and identity in self.members():
                        signal.pidfd_send_signal(fd,sig)
                finally:os.close(fd)
            except ProcessLookupError:pass
    def _reap(self):
        while True:
            try:
                pid,_=os.waitpid(-1,os.WNOHANG)
                if pid==0:return
            except ChildProcessError:return
    def close(self,grace=0.5,timeout=4):
        started=time.monotonic()
        self._signal(signal.SIGTERM)
        while time.monotonic()-started<grace:
            self._reap()
            if not self.members():return True
            time.sleep(0.02)
        while time.monotonic()-started<timeout:
            self._signal(signal.SIGKILL);self._reap()
            if not self.members():return True
            time.sleep(0.02)
        return False
