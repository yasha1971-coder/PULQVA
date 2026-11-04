//! Candidate non-admin Windows isolation primitive. Not enabled in GUI.
//! Raw anonymous pipes are IPC plumbing, NOT a llama-server inference protocol.
//! WIP: asset identity and profile-specific read ACL preparation are not wired.
//! Kept test-only until that API, bounded IPC and independent host acceptance exist.
//! No network capability, firewall rule or loopback exemption is requested.
#![cfg(windows)]
use std::{ffi::{c_void,OsString},fs::{self,File},io,mem::{size_of,zeroed},os::windows::{ffi::OsStrExt,fs::MetadataExt,io::FromRawHandle},path::Path,sync::atomic::{AtomicU64,Ordering},thread,time::{Duration,Instant,SystemTime,UNIX_EPOCH}};
type H=*mut c_void;
const INVALID:H=(-1isize) as H;
const WAIT_OBJECT_0:u32=0;
const WAIT_TIMEOUT:u32=258;
const HANDLE_FLAG_INHERIT:u32=1;
const JOB_KILL_ON_CLOSE:u32=0x2000;
const CREATE_SUSPENDED:u32=4;
const CREATE_UNICODE_ENVIRONMENT:u32=0x400;
const EXTENDED_STARTUPINFO_PRESENT:u32=0x80000;
const CREATE_NO_WINDOW:u32=0x08000000;
const STARTF_USESTDHANDLES:u32=0x100;
const SECURITY_CAPABILITIES:usize=0x20009;
const HANDLE_LIST:usize=0x20002;
static GENERATION:AtomicU64=AtomicU64::new(0);
#[repr(C)] struct SecurityAttributes{length:u32,descriptor:H,inherit:i32}
#[repr(C)] struct SecurityCapabilities{sid:H,capabilities:H,count:u32,reserved:u32}
#[repr(C)] struct StartupInfo{cb:u32,reserved:*mut u16,desktop:*mut u16,title:*mut u16,x:u32,y:u32,xsize:u32,ysize:u32,xcount:u32,ycount:u32,fill:u32,flags:u32,show:u16,reserved_len:u16,reserved_bytes:*mut u8,input:H,output:H,error:H}
#[repr(C)] struct StartupInfoEx{startup:StartupInfo,attributes:H}
#[repr(C)] struct ProcessInformation{process:H,thread:H,pid:u32,tid:u32}
#[repr(C)] struct BasicLimits{process_time:i64,job_time:i64,flags:u32,min_ws:usize,max_ws:usize,active:u32,affinity:usize,priority:u32,scheduling:u32}
#[repr(C)] struct IoCounters{read_ops:u64,write_ops:u64,other_ops:u64,read_bytes:u64,write_bytes:u64,other_bytes:u64}
#[repr(C)] struct ExtendedLimits{basic:BasicLimits,io:IoCounters,process_memory:usize,job_memory:usize,peak_process:usize,peak_job:usize}
#[repr(C)] struct Accounting{user:i64,kernel:i64,period_user:i64,period_kernel:i64,page_faults:u32,total:u32,active:u32,terminated:u32}
#[link(name="kernel32")] unsafe extern "system"{
 fn CloseHandle(h:H)->i32;
 fn GetDriveTypeW(root:*const u16)->u32;
 fn GetWindowsDirectoryW(buffer:*mut u16,size:u32)->u32;
 fn GetEnvironmentVariableW(name:*const u16,buffer:*mut u16,size:u32)->u32;
 fn CreatePipe(read:*mut H,write:*mut H,attrs:*mut SecurityAttributes,size:u32)->i32;
 fn SetHandleInformation(h:H,mask:u32,flags:u32)->i32;
 fn CreateJobObjectW(attrs:H,name:*const u16)->H;
 fn SetInformationJobObject(job:H,class:u32,info:H,len:u32)->i32;
 fn QueryInformationJobObject(job:H,class:u32,info:H,len:u32,returned:*mut u32)->i32;
 fn AssignProcessToJobObject(job:H,process:H)->i32;
 fn TerminateJobObject(job:H,exit:u32)->i32;
 fn InitializeProcThreadAttributeList(list:H,count:u32,flags:u32,size:*mut usize)->i32;
 fn UpdateProcThreadAttribute(list:H,flags:u32,key:usize,value:H,size:usize,previous:H,returned:*mut usize)->i32;
 fn DeleteProcThreadAttributeList(list:H);
 fn CreateProcessW(app:*const u16,command:*mut u16,process_attrs:H,thread_attrs:H,inherit:i32,flags:u32,environment:H,cwd:*const u16,startup:*mut StartupInfoEx,info:*mut ProcessInformation)->i32;
 fn ResumeThread(thread:H)->u32;
 fn TerminateProcess(process:H,exit:u32)->i32;
 fn WaitForSingleObject(handle:H,millis:u32)->u32;
 fn GetExitCodeProcess(process:H,code:*mut u32)->i32;
}
#[link(name="userenv")] unsafe extern "system"{
 fn CreateAppContainerProfile(name:*const u16,display:*const u16,description:*const u16,capabilities:H,count:u32,sid:*mut H)->i32;
 fn DeleteAppContainerProfile(name:*const u16)->i32;
}
#[link(name="advapi32")] unsafe extern "system"{fn FreeSid(sid:H)->H;}
fn last()->io::Error{io::Error::last_os_error()}
fn invalid(s:&str)->io::Error{io::Error::new(io::ErrorKind::InvalidInput,s)}
fn check(ok:i32)->io::Result<()>{if ok==0{Err(last())}else{Ok(())}}
struct Handle(H);
impl Handle{fn new(raw:H)->io::Result<Self>{if raw.is_null()||raw==INVALID{Err(last())}else{Ok(Self(raw))}}}
impl Drop for Handle{fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
struct Profile{name:Vec<u16>,sid:H,delete:bool}
impl Profile{
 fn create()->io::Result<Self>{
  let nanos=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|invalid("clock before epoch"))?.as_nanos();
  let name=wide(std::ffi::OsStr::new(&format!("PULQVA.Isolation.{}.{}.{}",std::process::id(),nanos,GENERATION.fetch_add(1,Ordering::Relaxed))))?;
  let mut sid=std::ptr::null_mut();
  let hr=unsafe{CreateAppContainerProfile(name.as_ptr(),name.as_ptr(),name.as_ptr(),std::ptr::null_mut(),0,&mut sid)};
  // Never reuse a foreign/pre-existing profile; every non-success is fail-closed.
  if hr<0{return Err(io::Error::new(io::ErrorKind::Other,format!("CreateAppContainerProfile HRESULT {hr:#x}")));}
  if sid.is_null(){unsafe{DeleteAppContainerProfile(name.as_ptr());}return Err(invalid("profile returned no SID"));}
  Ok(Self{name,sid,delete:true})
 }
}
impl Profile{
 fn cleanup(&mut self)->io::Result<()>{
  if self.delete{
   let hr=unsafe{DeleteAppContainerProfile(self.name.as_ptr())};
   if hr<0{return Err(io::Error::new(io::ErrorKind::Other,format!("DeleteAppContainerProfile HRESULT {hr:#x}")));}
   self.delete=false;
  }
  Ok(())
 }
}
impl Drop for Profile{fn drop(&mut self){let _=self.cleanup();unsafe{FreeSid(self.sid);}}}
struct Attributes{storage:Vec<usize>,initialized:bool}
impl Attributes{
 fn new()->io::Result<Self>{
  let mut bytes=0;
  unsafe{InitializeProcThreadAttributeList(std::ptr::null_mut(),2,0,&mut bytes);}
  if bytes==0||bytes>65536{return Err(last());}
  let mut value=Self{storage:vec![0;(bytes+size_of::<usize>()-1)/size_of::<usize>()],initialized:false};
  check(unsafe{InitializeProcThreadAttributeList(value.ptr(),2,0,&mut bytes)})?;
  value.initialized=true;Ok(value)
 }
 fn ptr(&mut self)->H{self.storage.as_mut_ptr().cast()}
 fn set<T>(&mut self,key:usize,value:&mut T)->io::Result<()>{check(unsafe{UpdateProcThreadAttribute(self.ptr(),0,key,(value as *mut T).cast(),size_of::<T>(),std::ptr::null_mut(),std::ptr::null_mut())})}
}
impl Drop for Attributes{fn drop(&mut self){if self.initialized{unsafe{DeleteProcThreadAttributeList(self.ptr());}}}}
fn wide(s:&std::ffi::OsStr)->io::Result<Vec<u16>>{
 let mut v:Vec<_>=s.encode_wide().collect();
 if v.contains(&0){return Err(invalid("embedded NUL"));}
 v.push(0);Ok(v)
}
/// A strictly minimal, explicit Unicode environment for a native Windows child.
/// Never inherit user PATH, proxy configuration, credentials, or other ambient vars.
/// Obtain SystemRoot from Windows itself, not from mutable parent environment.
/// An AppContainer profile depends on LOCALAPPDATA for startup. Pass only the
/// documented minimum, never PATH, proxies or the ambient environment.
/// See MicrosoftDocs/win32 implementing-an-appcontainer: Creating the Profile.
fn child_environment()->io::Result<Vec<u16>>{
 let mut root=[0u16;32768];
 let root_len=unsafe{GetWindowsDirectoryW(root.as_mut_ptr(),root.len() as u32)};
 if root_len==0{return Err(last());}
 if root_len as usize>=root.len(){return Err(invalid("Windows directory exceeds bound"));}
 if root_len<3||root[1]!=58||root[2]!=92||root[..root_len as usize].contains(&0){
  return Err(invalid("Windows directory is not a local absolute drive path"));
 }
 let name:Vec<u16>="LOCALAPPDATA\0".encode_utf16().collect();
 let mut local=[0u16;32768];
 let local_len=unsafe{GetEnvironmentVariableW(name.as_ptr(),local.as_mut_ptr(),local.len() as u32)};
 if local_len==0{
  return Err(io::Error::new(io::ErrorKind::NotFound,"LOCALAPPDATA is required for AppContainer profile initialization"));
 }
 if local_len as usize>=local.len(){return Err(invalid("LOCALAPPDATA exceeds bound"));}
 if local_len<3||!(local[0]>=65 && local[0]<=90 || local[0]>=97 && local[0]<=122)||local[1]!=58||local[2]!=92
   ||local[..local_len as usize].contains(&0) {
  return Err(invalid("LOCALAPPDATA must be an absolute local-drive path"));
 }
 let mut out:Vec<u16>="LOCALAPPDATA=".encode_utf16().collect();
 out.extend_from_slice(&local[..local_len as usize]);
 out.push(0);
 out.extend("SystemRoot=".encode_utf16());
 out.extend_from_slice(&root[..root_len as usize]);
 out.extend_from_slice(&[0,0]);
 Ok(out)
}
fn quote(s:&std::ffi::OsStr)->io::Result<Vec<u16>>{
 let mut raw=wide(s)?;raw.pop();
 let mut out=vec![34];let mut slashes=0;
 for c in raw{if c==92{slashes+=1;continue}
  if c==34{out.extend(std::iter::repeat_n(92,slashes*2+1));}else{out.extend(std::iter::repeat_n(92,slashes));}
  slashes=0;out.push(c);
 }
 out.extend(std::iter::repeat_n(92,slashes*2));out.push(34);Ok(out)
}
fn command(executable:&Path,args:&[OsString])->io::Result<Vec<u16>>{
 let mut out=quote(executable.as_os_str())?;
 for arg in args{out.push(32);out.extend(quote(arg.as_os_str())?);}
 if out.len()>=32767{return Err(invalid("command line exceeds Windows limit"));}
 out.push(0);Ok(out)
}
fn regular_path(path:&Path,directory:bool)->io::Result<()>{
 if !path.is_absolute(){return Err(invalid("absolute path required"));}
 match path.components().next(){
  Some(std::path::Component::Prefix(p)) if matches!(p.kind(),std::path::Prefix::Disk(_))=>{},
  _=>return Err(invalid("local drive path required")),
 }
 let prefix=path.as_os_str().encode_wide().take(3).collect::<Vec<_>>();
 if prefix.len()!=3||prefix[2]!=92{return Err(invalid("rooted drive path required"));}
 let root=[prefix[0],58,92,0];
 if unsafe{GetDriveTypeW(root.as_ptr())}!=3{return Err(invalid("fixed local drive required"));}
 for parent in path.ancestors(){
  let m=fs::symlink_metadata(parent)?;
  if m.file_attributes()&0x400!=0{return Err(invalid("reparse path rejected"));}
 }
 let m=fs::metadata(path)?;
 if if directory{!m.is_dir()}else{!m.is_file()}{return Err(invalid("wrong path kind"));}
 Ok(())
}
fn pipe()->io::Result<(Handle,Handle)>{
 let mut read=std::ptr::null_mut();let mut write=std::ptr::null_mut();
 let mut attrs=SecurityAttributes{length:size_of::<SecurityAttributes>() as u32,descriptor:std::ptr::null_mut(),inherit:1};
 check(unsafe{CreatePipe(&mut read,&mut write,&mut attrs,0)})?;
 Ok((Handle::new(read)?,Handle::new(write)?))
}
/// Owned zero-capability AppContainer child with private inherited pipe endpoints.
/// This is a candidate primitive, not qualification of DLLs, model, IPC protocol,
/// endpoint ownership, or actual IPv4/IPv6/DNS/proxy denial on a target machine.
pub struct IsolatedChild{
 process:Handle,job:Handle,profile:Option<Profile>,
 input:Option<File>,output:Option<File>,pid:u32,closed:bool,
}
impl IsolatedChild{
 /// No shell, no ambient environment and no inherited handles beyond two pipes.
 /// File access is fail-closed: this function does not broaden asset ACLs.
 pub fn spawn(executable:&Path,args:&[OsString],cwd:&Path)->io::Result<Self>{
  Self::spawn_inner(executable,args,cwd,None)
 }
 fn spawn_inner(executable:&Path,args:&[OsString],cwd:&Path,stderr:Option<H>)->io::Result<Self>{
  regular_path(executable,false)?;regular_path(cwd,true)?;
  let exe=wide(executable.as_os_str())?;let cwd=wide(cwd.as_os_str())?;
  let mut command=command(executable,args)?;
  let mut profile=Profile::create()?;
  let job=Handle::new(unsafe{CreateJobObjectW(std::ptr::null_mut(),std::ptr::null())})?;
  let mut limits:ExtendedLimits=unsafe{zeroed()};limits.basic.flags=JOB_KILL_ON_CLOSE;
  check(unsafe{SetInformationJobObject(job.0,9,(&mut limits as *mut ExtendedLimits).cast(),size_of::<ExtendedLimits>() as u32)})?;
  let (child_in,parent_in)=pipe()?;let (parent_out,child_out)=pipe()?;
  check(unsafe{SetHandleInformation(parent_in.0,HANDLE_FLAG_INHERIT,0)})?;
  check(unsafe{SetHandleInformation(parent_out.0,HANDLE_FLAG_INHERIT,0)})?;
  let mut caps=SecurityCapabilities{sid:profile.sid,capabilities:std::ptr::null_mut(),count:0,reserved:0};
  let mut inherited=vec![child_in.0,child_out.0];
  if let Some(h)=stderr{inherited.push(h);}
  let mut attributes=Attributes::new()?;
  attributes.set(SECURITY_CAPABILITIES,&mut caps)?;
  check(unsafe{UpdateProcThreadAttribute(attributes.ptr(),0,HANDLE_LIST,inherited.as_mut_ptr().cast(),inherited.len()*size_of::<H>(),std::ptr::null_mut(),std::ptr::null_mut())})?;
  let mut startup:StartupInfoEx=unsafe{zeroed()};
  startup.startup.cb=size_of::<StartupInfoEx>() as u32;
  startup.startup.flags=STARTF_USESTDHANDLES;
  startup.startup.input=child_in.0;startup.startup.output=child_out.0;startup.startup.error=stderr.unwrap_or(child_out.0);
  startup.attributes=attributes.ptr();
  let mut pi:ProcessInformation=unsafe{zeroed()};
  let mut environment=child_environment()?;
  check(unsafe{CreateProcessW(exe.as_ptr(),command.as_mut_ptr(),std::ptr::null_mut(),std::ptr::null_mut(),1,
   CREATE_SUSPENDED|CREATE_UNICODE_ENVIRONMENT|EXTENDED_STARTUPINFO_PRESENT|CREATE_NO_WINDOW,
   environment.as_mut_ptr().cast(),cwd.as_ptr(),&mut startup,&mut pi)})
   .map_err(|e|io::Error::new(e.kind(),format!("CreateProcessW failed: {e}")))?;
  let process=Handle::new(pi.process)?;let primary=Handle::new(pi.thread)?;
  if let Err(e)=check(unsafe{AssignProcessToJobObject(job.0,process.0)}){
   let terminated=unsafe{TerminateProcess(process.0,1)};
   let waited=unsafe{WaitForSingleObject(process.0,5000)};
   if terminated==0||waited!=WAIT_OBJECT_0{
    profile.delete=false;
    return Err(io::Error::new(io::ErrorKind::Other,"assignment failed; suspended child termination unconfirmed"));
   }
   return Err(e);
  }
  // The child cannot execute user code or create descendants before Job assignment.
  if unsafe{ResumeThread(primary.0)}==u32::MAX{
   let e=last();
   let terminated=unsafe{TerminateJobObject(job.0,1)};
   let waited=unsafe{WaitForSingleObject(process.0,5000)};
   if terminated==0||waited!=WAIT_OBJECT_0{
    profile.delete=false;
    return Err(io::Error::new(io::ErrorKind::Other,"resume failed; job termination unconfirmed"));
   }
   return Err(e);
  }
  drop(primary);drop(child_in);drop(child_out);
  let input=unsafe{File::from_raw_handle(parent_in.0)};
  std::mem::forget(parent_in);
  let output=unsafe{File::from_raw_handle(parent_out.0)};
  std::mem::forget(parent_out);
  Ok(Self{process,job,profile:Some(profile),input:Some(input),output:Some(output),pid:pi.pid,closed:false})
 }
 pub fn pid(&self)->u32{self.pid}
 /// Blocking pipe primitives. Higher-level framing/deadlines are NOT implemented.
 pub fn input(&mut self)->io::Result<&mut File>{self.input.as_mut().ok_or_else(||invalid("closed"))}
 pub fn output(&mut self)->io::Result<&mut File>{self.output.as_mut().ok_or_else(||invalid("closed"))}
 pub fn try_exit(&self)->io::Result<Option<u32>>{
  let wait=unsafe{WaitForSingleObject(self.process.0,0)};
  if wait==WAIT_TIMEOUT{return Ok(None);}
  if wait!=WAIT_OBJECT_0{return Err(last());}
  let mut code=0;check(unsafe{GetExitCodeProcess(self.process.0,&mut code)})?;Ok(Some(code))
 }
 pub fn close(&mut self)->io::Result<()>{
  if self.closed{return Ok(());}
  self.input.take();self.output.take();
  check(unsafe{TerminateJobObject(self.job.0,1)})?;
  let deadline=Instant::now()+Duration::from_secs(5);
  loop{
   let mut accounting:Accounting=unsafe{zeroed()};
   check(unsafe{QueryInformationJobObject(self.job.0,1,(&mut accounting as *mut Accounting).cast(),size_of::<Accounting>() as u32,std::ptr::null_mut())})?;
   if accounting.active==0{if let Some(p)=self.profile.as_mut(){p.cleanup()?;}self.closed=true;self.profile.take();return Ok(());}
   if Instant::now()>=deadline{
    // Do not delete a profile while a child could still use it.
    if let Some(p)=self.profile.as_mut(){p.delete=false;}
    return Err(io::Error::new(io::ErrorKind::TimedOut,"job cleanup not confirmed"));
   }
   thread::sleep(Duration::from_millis(10));
  }
 }
}
impl Drop for IsolatedChild{fn drop(&mut self){if self.close().is_err(){if let Some(p)=self.profile.as_mut(){p.delete=false;}}}}
#[cfg(test)] mod tests{
 use super::*;
 #[test]fn quoting_is_literal_and_handles_slashes(){
  assert_eq!(String::from_utf16(&quote(std::ffi::OsStr::new("a b")).unwrap()).unwrap(),"\"a b\"");
  assert_eq!(String::from_utf16(&quote(std::ffi::OsStr::new("a\\\"b\\")).unwrap()).unwrap(),"\"a\\\\\\\"b\\\\\"");
 }
 #[test]fn rejects_nul_before_any_process(){
  assert!(quote(std::ffi::OsStr::new("a\0b")).is_err());
 }
 #[test]fn rejects_relative_paths_without_creating_profile(){
  assert!(regular_path(Path::new("relative.exe"),false).is_err());
 }
 #[test]fn rejects_unc_without_filesystem_or_network_access(){
  assert!(regular_path(Path::new(r"\\server\share\app.exe"),false).is_err());
 }
 #[test]fn environment_contains_only_required_appcontainer_variables(){
  let env=child_environment().expect("Windows must provide LOCALAPPDATA and system root");
  assert!(env.ends_with(&[0,0]));
  let entries:Vec<_>=env.split(|c|*c==0).filter(|e|!e.is_empty())
    .map(|e|String::from_utf16(e).unwrap()).collect();
  assert_eq!(entries.len(),2);
  assert!(entries[0].starts_with("LOCALAPPDATA="));
  assert!(entries[1].starts_with("SystemRoot="));
  assert!(entries.iter().all(|e|!e.starts_with("PATH=") && !e.contains("PROXY")
     && !e.starts_with("USERPROFILE=")));
 }
 #[test]fn rejects_oversized_command(){
  assert!(command(Path::new("C:\\x.exe"),&[OsString::from("x".repeat(32768))]).is_err());
 }

 // WIN-ISOLATION-02-PROBE — manual Windows execution only. This is a
 // real AppContainer stdio probe, not a real-Qwen or egress-denial test.
 const IPC_CHALLENGE: &[u8; 5] = b"PING\n";
 const IPC_RESPONSE: &[u8] = b"PULQVA_ISOLATION_IPC_V1:PONG\n";

 #[test]
 fn isolated_ipc_child_witness() {
  use std::io::{Read,Write};
  // The exact test filter is present only in the spawned child invocation.
  // The ordinary test suite returns without reading stdin.
  if !std::env::args().any(|s|s=="windows_isolation::tests::isolated_ipc_child_witness"){return;}
  let mut challenge=[0u8;5];
  if std::io::stdin().read_exact(&mut challenge).is_ok() && &challenge==IPC_CHALLENGE {
   let mut stdout=std::io::stdout().lock();
   let _=stdout.write_all(IPC_RESPONSE);
   let _=stdout.flush();
  }
 }

 #[test]
 #[ignore = "requires explicitly authorized real Windows AppContainer test; not CI acceptance"]
 fn real_appcontainer_stdio_roundtrip_probe() -> io::Result<()> {
  use std::io::{Read,Write};
  use std::sync::mpsc;
  let exe=std::env::current_exe()?;
  let cwd=exe.parent().ok_or_else(||invalid("test exe has no parent"))?;
  let args=[
   OsString::from("--exact"),
   OsString::from("windows_isolation::tests::isolated_ipc_child_witness"),
   OsString::from("--nocapture"),
  ];
  let mut child=IsolatedChild::spawn(&exe,&args,cwd)?;
  child.input()?.write_all(IPC_CHALLENGE)?;
  child.input()?.flush()?;
  let mut output=child.output()?.try_clone()?;
  let (tx,rx)=mpsc::sync_channel(1);
  let worker=std::thread::spawn(move || {
   let mut buf=[0u8;1024];let mut received=Vec::new();
   loop {
    match output.read(&mut buf) {
     Ok(0)=>break,
     Ok(n)=>{
      if received.len()+n>4096 {break;}
      received.extend_from_slice(&buf[..n]);
     },
     Err(_)=>break,
    }
   }
   let _=tx.send(received);
  });
  let response=rx.recv_timeout(Duration::from_secs(8));
  let cleanup=child.close();
  if response.is_ok() && cleanup.is_ok(){let _=worker.join();}
  cleanup?;
  let bytes=response.map_err(|_|io::Error::new(io::ErrorKind::TimedOut,"AppContainer stdio witness timed out"))?;
  if !bytes.windows(IPC_RESPONSE.len()).any(|window|window==IPC_RESPONSE) {
   return Err(io::Error::new(io::ErrorKind::InvalidData,
     "AppContainer child returned no valid bounded IPC witness"));
  }
  Ok(())
 }
}

#[cfg(test)]
#[path = "model_ipc_witness.rs"]
mod model_ipc_witness;
