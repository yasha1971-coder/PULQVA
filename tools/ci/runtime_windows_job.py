"""Native Windows owned-job primitive for bounded diagnostic fixtures.

No PULQVA/Tor policy or UI actions. Processes join the job at creation;
all handles are non-inheritable and breakaway is disabled.
"""
import ctypes as C
from ctypes import wintypes as W
import os
import subprocess
import threading
import time

class WindowsJob:
    def __init__(self):
        if os.name!='nt':raise OSError('Windows required')
        self.k=C.WinDLL('kernel32',use_last_error=True)
        self.handle=None;self.timer=None;self.lock=threading.RLock()
        self.configure_api()
        self.handle=self.k.CreateJobObjectW(None,None)
        if not self.handle:raise C.WinError(C.get_last_error())
        try:
            info=Extended();info.Basic.Flags=0x2000
            self.check(self.k.SetInformationJobObject(self.handle,9,C.byref(info),C.sizeof(info)))
        except BaseException:
            self.k.CloseHandle(self.handle);self.handle=None;raise

    def configure_api(self):
        defs={
            'CreateJobObjectW':([C.c_void_p,W.LPCWSTR],W.HANDLE),
            'SetInformationJobObject':([W.HANDLE,C.c_int,C.c_void_p,W.DWORD],W.BOOL),
            'QueryInformationJobObject':([W.HANDLE,C.c_int,C.c_void_p,W.DWORD,C.c_void_p],W.BOOL),
            'TerminateJobObject':([W.HANDLE,W.UINT],W.BOOL),
            'CloseHandle':([W.HANDLE],W.BOOL),
            'InitializeProcThreadAttributeList':([C.c_void_p,W.DWORD,W.DWORD,C.POINTER(C.c_size_t)],W.BOOL),
            'UpdateProcThreadAttribute':([C.c_void_p,W.DWORD,C.c_size_t,C.c_void_p,C.c_size_t,C.c_void_p,C.c_void_p],W.BOOL),
            'DeleteProcThreadAttributeList':([C.c_void_p],None),
            'CreateProcessW':([W.LPCWSTR,W.LPWSTR,C.c_void_p,C.c_void_p,W.BOOL,W.DWORD,C.c_void_p,W.LPCWSTR,C.c_void_p,C.POINTER(ProcessInfo)],W.BOOL)}
        for name,(args,result) in defs.items():
            f=getattr(self.k,name);f.argtypes=args;f.restype=result

    @staticmethod
    def check(ok):
        if not ok:raise C.WinError(C.get_last_error())

    def launch(self, argv, cwd):
        if not argv or not os.path.isabs(argv[0]):raise ValueError('absolute executable required')
        with self.lock:
            if not self.handle:raise RuntimeError('job closed')
            size=C.c_size_t()
            self.k.InitializeProcThreadAttributeList(None,1,0,C.byref(size))
            attrs=C.create_string_buffer(size.value)
            self.check(self.k.InitializeProcThreadAttributeList(attrs,1,0,C.byref(size)))
            jobs=(W.HANDLE*1)(self.handle)
            try:
                self.check(self.k.UpdateProcThreadAttribute(attrs,0,0x2000D,
                           C.cast(jobs,C.c_void_p),C.sizeof(jobs),None,None))
                si=StartupEx();si.Startup.cb=C.sizeof(si);si.Attributes=C.cast(attrs,C.c_void_p)
                pi=ProcessInfo()
                cmd=C.create_unicode_buffer(subprocess.list2cmdline(argv))
                self.check(self.k.CreateProcessW(argv[0],cmd,None,None,False,
                           0x80000|0x08000000,None,str(cwd),C.byref(si),C.byref(pi)))
                try:return pi.Pid
                finally:
                    self.k.CloseHandle(pi.Thread);self.k.CloseHandle(pi.Process)
            finally:self.k.DeleteProcThreadAttributeList(attrs)

    def members(self):
        with self.lock:
            if not self.handle:raise RuntimeError('job closed')
            # Fail on overflow, never truncate an ownership inventory.
            buf=C.create_string_buffer(8+1024*C.sizeof(C.c_size_t))
            self.check(self.k.QueryInformationJobObject(self.handle,3,buf,C.sizeof(buf),None))
            count=C.cast(C.addressof(buf)+4,C.POINTER(W.DWORD)).contents.value
            if count>1024:raise RuntimeError('job inventory overflow')
            ids=(C.c_size_t*count).from_address(C.addressof(buf)+8)
            return tuple(ids)

    def terminate_owned_job(self):
        with self.lock:
            if self.handle:self.check(self.k.TerminateJobObject(self.handle,1))

    def arm_deadline(self, absolute_monotonic):
        remaining=absolute_monotonic-time.monotonic()
        if remaining<=0:raise ValueError('expired deadline')
        if self.timer is not None:raise RuntimeError('deadline already armed')
        self.timer=threading.Timer(remaining,self.terminate_owned_job)
        self.timer.daemon=True;self.timer.start()

    def cleanup(self, timeout=5):
        if self.timer:self.timer.cancel()
        self.terminate_owned_job()
        deadline=time.monotonic()+timeout
        while self.members():
            if time.monotonic()>=deadline:raise TimeoutError('owned job not empty')
            time.sleep(.01)
        return True

    def close(self):
        with self.lock:
            if self.timer:self.timer.cancel()
            if self.handle:
                self.check(self.k.CloseHandle(self.handle))
                self.handle=None  # kill-on-close remains emergency fallback

    def __enter__(self):return self
    def __exit__(self,*exc):
        try:self.cleanup()
        finally:self.close()

class Basic(C.Structure):
    _fields_=[('ProcessTime',C.c_int64),('JobTime',C.c_int64),('Flags',W.DWORD),
              ('Min',C.c_size_t),('Max',C.c_size_t),('ActiveLimit',W.DWORD),
              ('Affinity',C.c_size_t),('Priority',W.DWORD),('Scheduling',W.DWORD)]
class IO(C.Structure):
    _fields_=[(n,C.c_uint64) for n in ('ReadOps','WriteOps','OtherOps','ReadBytes','WriteBytes','OtherBytes')]
class Extended(C.Structure):
    _fields_=[('Basic',Basic),('IO',IO),('ProcessMemory',C.c_size_t),
              ('JobMemory',C.c_size_t),('PeakProcess',C.c_size_t),('PeakJob',C.c_size_t)]
class Startup(C.Structure):
    _fields_=[('cb',W.DWORD),('Reserved',W.LPWSTR),('Desktop',W.LPWSTR),('Title',W.LPWSTR),
              *[(n,W.DWORD) for n in ('X','Y','XSize','YSize','XChars','YChars','Fill','Flags')],
              ('Show',W.WORD),('ReservedBytes',W.WORD),('ReservedPointer',C.c_void_p),
              ('Input',W.HANDLE),('Output',W.HANDLE),('Error',W.HANDLE)]
class StartupEx(C.Structure):
    _fields_=[('Startup',Startup),('Attributes',C.c_void_p)]
class ProcessInfo(C.Structure):
    _fields_=[('Process',W.HANDLE),('Thread',W.HANDLE),('Pid',W.DWORD),('Tid',W.DWORD)]

