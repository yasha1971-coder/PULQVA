use super::*;
use std::collections::BTreeSet;
use std::io::Read;
use std::os::windows::io::AsRawHandle;
use std::path::PathBuf;
const CAP:usize=1048576;

#[link(name="kernel32")] unsafe extern "system" {
    fn PeekNamedPipe(pipe:H,buffer:H,size:u32,read:*mut u32,available:*mut u32,left:*mut u32)->i32;
    fn OpenProcess(access:u32,inherit:i32,pid:u32)->H;
    fn IsProcessInJob(process:H,job:H,result:*mut i32)->i32;
    fn QueryFullProcessImageNameW(process:H,flags:u32,name:*mut u16,len:*mut u32)->i32;
}
#[link(name="advapi32")] unsafe extern "system" {
    fn OpenProcessToken(process:H,access:u32,token:*mut H)->i32;
    fn GetTokenInformation(token:H,kind:u32,buffer:H,size:u32,returned:*mut u32)->i32;
    fn EqualSid(left:H,right:H)->i32;
}

fn drain(f:&mut File,out:&mut Vec<u8>,remaining:usize)->io::Result<bool>{
 let mut available=0;
 if unsafe{PeekNamedPipe(f.as_raw_handle(),std::ptr::null_mut(),0,std::ptr::null_mut(),&mut available,std::ptr::null_mut())}==0{
  let e=last();if e.raw_os_error()==Some(109){return Ok(true);}return Err(e);
 }
 if available as usize>remaining{return Err(invalid("combined output cap exceeded"));}
 if available>0{
  let mut buf=vec![0;available as usize];f.read_exact(&mut buf)?;out.extend(buf);
 }
 Ok(false)
}

fn system32()->io::Result<PathBuf>{
    let mut buf=[0u16;32768];
    let n=unsafe{GetWindowsDirectoryW(buf.as_mut_ptr(),buf.len() as u32)} as usize;
    if n==0 || n>=buf.len(){return Err(last());}
    Ok(PathBuf::from(String::from_utf16_lossy(&buf[..n])).join("System32"))
}

pub(super) fn members(job:H)->io::Result<Vec<u32>>{
    let mut buf=[0usize;65];
    check(unsafe{QueryInformationJobObject(job,3,buf.as_mut_ptr().cast(),size_of_val(&buf) as u32,std::ptr::null_mut())})?;
    let header=buf.as_ptr().cast::<u32>();
    let assigned=unsafe{*header} as usize;
    let listed=unsafe{*header.add(1)} as usize;
    if assigned!=listed || listed>=buf.len(){return Err(invalid("Incomplete Job PID list"));}
    buf[1..1+listed].iter().map(|&pid|u32::try_from(pid).map_err(|_|invalid("Invalid Job PID"))).collect()
}

fn token_info(token:H,kind:u32)->io::Result<Vec<usize>>{
    let mut len=0;
    unsafe{GetTokenInformation(token,kind,std::ptr::null_mut(),0,&mut len);}
    if len==0 || len>65536{return Err(last());}
    let mut value=vec![0usize;(len as usize).div_ceil(size_of::<usize>())];
    check(unsafe{GetTokenInformation(token,kind,value.as_mut_ptr().cast(),len,&mut len)})?;
    Ok(value)
}

fn verify_job_member(job:H,pid:u32,exe:&Path,sid:H)->io::Result<()>{
    let process=Handle::new(unsafe{OpenProcess(0x1000,0,pid)})?;
    let mut inside=0;
    check(unsafe{IsProcessInJob(process.0,job,&mut inside)})?;
    if inside==0{return Err(invalid("PID lost verified Job membership"));}
    let mut buf=[0u16;32768];
    let mut n=buf.len() as u32;
    check(unsafe{QueryFullProcessImageNameW(process.0,0,buf.as_mut_ptr(),&mut n)})?;
    let path=String::from_utf16_lossy(&buf[..n as usize]);
    if !path.eq_ignore_ascii_case(&exe.to_string_lossy()){
        return Err(invalid("Unexpected Job member image"));
    }
    let mut raw=std::ptr::null_mut();
    check(unsafe{OpenProcessToken(process.0,8,&mut raw)})?;
    let token=Handle::new(raw)?;
    let is_ac=token_info(token.0,29)?[0] as u32;
    let capabilities=token_info(token.0,30)?[0] as u32;
    let ac=token_info(token.0,31)?;
    if is_ac!=1 || capabilities!=0 || ac[0]==0 || unsafe{EqualSid(ac[0] as H,sid)}==0{
        return Err(invalid("Unexpected Job member token"));
    }
    Ok(())
}

fn observe(child:&IsolatedChild,exe:&Path,console:&Path,seen:&mut BTreeSet<u32>)->io::Result<()>{
    let sid=child.profile.as_ref().ok_or_else(||invalid("Missing profile"))?.sid;
    for pid in members(child.job.0)?{
        if seen.contains(&pid){continue;}
        verify_job_member(child.job.0,pid,if pid==child.pid(){exe}else{console},sid)?;
        seen.insert(pid);
    }
    Ok(())
}


/// Launches a prepared local executable under an owned Job and a fresh
/// zero-capability profile. The caller supplies trusted assets and arguments.
fn collect_completion(executable: &Path, args: &[OsString], cwd: &Path,
    prepare: impl FnOnce(H) -> io::Result<()>) -> io::Result<Vec<u8>> {
    let conhost = system32()?.join("conhost.exe");
    let (reader, writer) = pipe()?;
    check(unsafe { SetHandleInformation(reader.0, HANDLE_FLAG_INHERIT, 0) })?;
    let mut stderr = unsafe { File::from_raw_handle(reader.0) };
    std::mem::forget(reader);
    let deadline = Instant::now() + Duration::from_secs(95);
    let mut child = IsolatedChild::spawn_prepared(executable, args, cwd, Some(writer.0), prepare)?;
    drop(writer);
    child.input.take();
    let mut stdout = child.output()?.try_clone()?;
    let (mut output, mut diagnostics) = (Vec::new(), Vec::new());
    let mut observed = BTreeSet::new();

    let collect = (|| -> io::Result<()> {
        loop {
            if Instant::now() >= deadline {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Local completion deadline"));
            }
            observe(&child, executable, &conhost, &mut observed)?;
            let left = CAP - output.len() - diagnostics.len();
            drain(&mut stdout, &mut output, left)?;
            let left = CAP - output.len() - diagnostics.len();
            drain(&mut stderr, &mut diagnostics, left)?;
            if let Some(code) = child.try_exit()? {
                if code != 0 {
                    return Err(invalid("Local completion process failed"));
                }
                observe(&child, executable, &conhost, &mut observed)?;
                return Ok(());
            }
            thread::sleep(Duration::from_millis(10));
        }
    })();

    // No successful result can escape a failed cleanup, including on timeout.
    child.close()?;
    collect?;
    let mut accounting: Accounting = unsafe { zeroed() };
    check(unsafe {
        QueryInformationJobObject(child.job.0, 1, (&mut accounting as *mut Accounting).cast(),
            size_of::<Accounting>() as u32, std::ptr::null_mut())
    })?;
    if accounting.active != 0 || accounting.total as usize != observed.len()
        || !observed.contains(&child.pid())
    {
        return Err(invalid("Job accounting differs from verified processes"));
    }

    let eof_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let left = CAP - output.len() - diagnostics.len();
        let stdout_eof = drain(&mut stdout, &mut output, left)?;
        let left = CAP - output.len() - diagnostics.len();
        let stderr_eof = drain(&mut stderr, &mut diagnostics, left)?;
        if stdout_eof && stderr_eof {
            return Ok(output);
        }
        if Instant::now() >= eof_deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "Missing pipe EOF"));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
#[test]
fn pipe_cap_and_eof_are_enforced_without_launching_a_process() -> io::Result<()> {
    use std::io::Write;

    let (reader, writer) = pipe()?;
    let mut reader_file = unsafe { File::from_raw_handle(reader.0) };
    let mut writer_file = unsafe { File::from_raw_handle(writer.0) };
    std::mem::forget(reader);
    std::mem::forget(writer);

    writer_file.write_all(b"fixture")?;
    let mut output = Vec::new();
    assert!(drain(&mut reader_file, &mut output, 6).is_err());
    assert!(output.is_empty());
    assert!(!drain(&mut reader_file, &mut output, 7)?);
    assert_eq!(output, b"fixture");
    drop(writer_file);
    assert!(drain(&mut reader_file, &mut output, 0)?);

    let job = Handle::new(unsafe { CreateJobObjectW(std::ptr::null_mut(), std::ptr::null_mut()) })?;
    assert!(members(job.0)?.is_empty());
    Ok(())
}

use pulqva_core::Interpretation;
use pulqva_intent_json::parse_interpretation_json;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::os::windows::fs::OpenOptionsExt;

const SYSTEM_POLICY: &str = include_str!("../../../sidecars/llama.cpp/intent.system.txt");
const SCHEMA: &str = include_str!("../../../sidecars/llama.cpp/intent.schema.json");
const WORKER: &str = "llama-app.exe";
const MODEL: &str = "model.gguf";
const REQUEST: &str = "request.bin";
const PRIVATE_DACL: &str = "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;OW)";
const COMPLETION_END: &str = " [end of text]\r\n\r\n\r\n";

#[link(name = "advapi32")]
unsafe extern "system" {
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(text: *const u16, revision: u32,
        descriptor: *mut H, size: *mut u32) -> i32;
    fn GetSecurityDescriptorDacl(descriptor: H, present: *mut i32, acl: *mut H,
        defaulted: *mut i32) -> i32;
    fn SetNamedSecurityInfoW(path: *mut u16, object_type: u32, info: u32,
        owner: H, group: H, dacl: H, sacl: H) -> u32;
    fn ConvertSidToStringSidW(sid: H, text: *mut *mut u16) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateDirectoryW(path: *const u16, attrs: *const SecurityAttributes) -> i32;
    fn LocalFree(memory: H) -> H;
}

struct LocalAllocation(H);
impl Drop for LocalAllocation {
    fn drop(&mut self) { unsafe { LocalFree(self.0); } }
}
fn descriptor(sddl: &str) -> io::Result<LocalAllocation> {
    let sddl = wide(std::ffi::OsStr::new(sddl))?;
    let mut raw = std::ptr::null_mut();
    check(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl.as_ptr(), 1, &mut raw, std::ptr::null_mut())
    })?;
    Ok(LocalAllocation(raw))
}

struct Scratch(PathBuf);
impl Scratch {
    fn create(parent: &Path) -> io::Result<Self> {
        regular_path(parent, true)?;
        let tick = SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_nanos();
        let generation = GENERATION.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!("pulqva-intent-{}-{tick}-{generation}", std::process::id()));
        let native_path = wide(path.as_os_str())?;
        let security = descriptor(PRIVATE_DACL)?;
        let attrs = SecurityAttributes {
            length: size_of::<SecurityAttributes>() as u32, descriptor: security.0, inherit: 0,
        };
        check(unsafe { CreateDirectoryW(native_path.as_ptr(), &attrs) })?;
        Ok(Self(path))
    }
    fn close(&self) -> io::Result<()> {
        fs::remove_dir_all(&self.0)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) { let _ = self.close(); }
}

fn grant_profile_read(root: &Path, sid: H) -> io::Result<()> {
    let mut text = std::ptr::null_mut();
    check(unsafe { ConvertSidToStringSidW(sid, &mut text) })?;
    let _allocated = LocalAllocation(text.cast());
    let mut len = 0;
    while unsafe { *text.add(len) } != 0 { len += 1; }
    let sid = String::from_utf16(unsafe { std::slice::from_raw_parts(text, len) }).map_err(io::Error::other)?;
    if !sid.starts_with("S-1-15-2-") {
        return Err(invalid("Unexpected local model profile SID"));
    }
    for (path, rights) in [(root.to_owned(), "GRGX"), (root.join(WORKER), "GRGX"),
        (root.join(MODEL), "GR"), (root.join(REQUEST), "GR")] {
        let mut path = wide(path.as_os_str())?;
        let security = descriptor(&format!("{PRIVATE_DACL}(A;;{rights};;;{sid})"))?;
        let mut acl = std::ptr::null_mut();
        let (mut present, mut defaulted) = (0, 0);
        check(unsafe { GetSecurityDescriptorDacl(security.0, &mut present, &mut acl, &mut defaulted) })?;
        if present == 0 || acl.is_null() { return Err(invalid("Missing private DACL")); }
        // SE_FILE_OBJECT, protected DACL only.
        let status = unsafe { SetNamedSecurityInfoW(path.as_mut_ptr(), 1, 0x8000_0004,
            std::ptr::null_mut(), std::ptr::null_mut(), acl, std::ptr::null_mut()) };
        if status != 0 { return Err(io::Error::from_raw_os_error(status as i32)); }
    }
    Ok(())
}

fn read_locked(path: &Path) -> io::Result<File> {
    regular_path(path, false)?;
    // Allow reads from the model process, deny replacement and write access.
    fs::OpenOptions::new().read(true).share_mode(1).open(path)
}
fn copy_pinned(source: &Path, destination: &Path, size: u64, sha256: &str) -> io::Result<File> {
    let mut source = read_locked(source)?;
    if source.metadata()?.len() != size {
        return Err(invalid("Bundled local model asset length mismatch"));
    }
    let mut destination_file = fs::OpenOptions::new().write(true).create_new(true).open(destination)?;
    if io::copy(&mut source, &mut destination_file)? != size {
        return Err(invalid("Incomplete local model asset copy"));
    }
    destination_file.sync_all()?;
    drop(destination_file);

    let mut locked = read_locked(destination)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = locked.read(&mut buffer)?;
        if count == 0 { break; }
        digest.update(&buffer[..count]);
    }
    let actual: String = digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    if actual != sha256 {
        return Err(invalid("Bundled local model asset SHA-256 mismatch"));
    }
    Ok(locked)
}

#[derive(Debug)]
pub enum BundledIntentError { Io(io::Error), InvalidOutput }

fn decode_output(stdout: &[u8]) -> Result<Interpretation, BundledIntentError> {
    let text = std::str::from_utf8(stdout).map_err(|_| BundledIntentError::InvalidOutput)?;
    let text = text.strip_suffix(COMPLETION_END).unwrap_or(text);
    parse_interpretation_json(text).map_err(|_| BundledIntentError::InvalidOutput)
}

/// Runs the pinned offline completion worker without a listening socket.
pub fn interpret_bundled(resource_root: &Path, private_parent: &Path, query: &str)
    -> Result<Interpretation, BundledIntentError>
{
    if query.trim().is_empty() || query.len() > 4096 || query.contains('\0') {
        return Err(BundledIntentError::Io(invalid("Invalid local request")));
    }
    let assets = resource_root.join("sidecars/llama.cpp/windows-x86_64");
    let scratch = Scratch::create(private_parent).map_err(BundledIntentError::Io)?;
    let execute = || -> io::Result<Vec<u8>> {
        let executable = scratch.0.join(WORKER);
        let _exe = copy_pinned(&assets.join(WORKER), &executable, 17_158_328,
            "50a2f85c09ece29360288ed2a753e046af7f28b54bb89c53da5de4264edd7d55")?;
        let _model = copy_pinned(&assets.join(MODEL), &scratch.0.join(MODEL), 3_013_027_808,
            "13c16f426047e2de38cd075bdade4a7bcbc8c774384876f677740cda65f8a983")?;
        let request = scratch.0.join(REQUEST);
        let mut request_file = fs::OpenOptions::new().write(true).create_new(true).open(&request)?;
        request_file.write_all(query.as_bytes())?;
        drop(request_file);
        let _request = read_locked(&request)?;
        let args = ["completion", "-m", MODEL, "--json-schema", SCHEMA, "--system-prompt",
            SYSTEM_POLICY, "--single-turn", "--simple-io", "--no-display-prompt",
            "--binary-file", REQUEST, "--no-escape", "--offline", "-n", "512", "-c", "4096",
            "-t", "4", "--no-warmup"].into_iter().map(OsString::from).collect::<Vec<_>>();
        collect_completion(&executable, &args, &scratch.0, |sid| grant_profile_read(&scratch.0, sid))
    };
    let bytes = execute();
    scratch.close().map_err(BundledIntentError::Io)?;
    decode_output(&bytes.map_err(BundledIntentError::Io)?)
}

#[test]
fn completion_accepts_only_the_existing_validated_format() {
    let json = r#"{"kind":"intent","query":"Big Buck Bunny","choice_mode":"ask"}"#;
    assert!(matches!(decode_output(format!("{json}{COMPLETION_END}").as_bytes()), Ok(Interpretation::Intent(_))));
    assert!(matches!(decode_output(br#"{"kind":"reject","reason":"semantic_authority"}"#), Ok(Interpretation::Reject(_))));
    for invalid in ["garbage", r#"{"kind":"intent","query":"Big Buck Bunny","choice_mode":"ask","url":"https://example.org"}"#] {
        assert!(matches!(decode_output(invalid.as_bytes()), Err(BundledIntentError::InvalidOutput)));
    }
}

