//! Test-only fixed-public MODEL-IPC-01. Never called by the product.
use super::*;
use std::io::Read;
use std::os::windows::io::AsRawHandle;
use sha2::{Digest,Sha256};
use serde_json::json;
const PROMPT:&str="Find a countdown video. Ask me to choose a result.";
const SCHEMA:&str=include_str!("../../../sidecars/llama.cpp/intent.schema.json");
const SYSTEM:&str=include_str!("../../../sidecars/llama.cpp/intent.system.txt");
const CAP:usize=1048576;
#[link(name="kernel32")] unsafe extern "system" {
 fn PeekNamedPipe(pipe:H,buffer:H,size:u32,read:*mut u32,available:*mut u32,left:*mut u32)->i32;
}
fn hex(bytes:&[u8])->String{
 const DIGITS:&[u8;16]=b"0123456789abcdef";
 let mut encoded=String::with_capacity(bytes.len()*2);
 for &byte in bytes{
  encoded.push(DIGITS[(byte>>4) as usize] as char);
  encoded.push(DIGITS[(byte&15) as usize] as char);
 }
 encoded
}
fn digest(bytes:&[u8])->String{hex(&Sha256::digest(bytes))}
fn hash_file(path:&Path)->io::Result<(u64,String)>{
 regular_path(path,false)?;
 let mut f=File::open(path)?;let mut h=Sha256::new();let mut size=0;
 let mut buf=[0u8;65536];
 loop{let n=f.read(&mut buf)?;if n==0{break;}h.update(&buf[..n]);size+=n as u64;}
 Ok((size,hex(&h.finalize())))
}
fn env_path(key:&str)->io::Result<std::path::PathBuf>{
 std::env::var_os(key).map(Into::into).ok_or_else(||invalid(key))
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
fn decode(bytes:&[u8])->io::Result<()>{
 let text=std::str::from_utf8(bytes).map_err(|_|invalid("stdout is not UTF-8"))?;
 let v=pulqva_intent_json::parse_interpretation_json(text).map_err(|e|invalid(&e.to_string()))?;
 match v{
  pulqva_core::Interpretation::Intent(i) if i.choice_mode()==pulqva_core::ChoiceMode::Ask=>Ok(()),
  _=>Err(invalid("fixed public request requires intent/ask")),
 }
}
// Inline native arguments remove the child's dependency on temporary policy files.
// Preserve exact compiled strings; no shell escaping or permission change.
fn model_args(model:&Path)->Vec<OsString>{
 vec![
  "-m".into(),model.as_os_str().to_owned(),"--json-schema".into(),SCHEMA.into(),
  "--system-prompt".into(),SYSTEM.into(),"--single-turn".into(),
  "--simple-io".into(),"--no-display-prompt".into(),"-p".into(),PROMPT.into(),
  "-n".into(),"512".into(),"-c".into(),"4096".into(),"-t".into(),"4".into(),"--no-warmup".into()]
}
#[test]fn inline_policy_survives_actual_windows_argument_parsing(){
 use std::os::windows::ffi::OsStrExt;
 #[link(name="shell32")] unsafe extern "system"{
  fn CommandLineToArgvW(line:*const u16,count:*mut i32)->*mut *mut u16;
 }
 #[link(name="kernel32")] unsafe extern "system"{fn LocalFree(value:H)->H;}
 struct Parsed(*mut *mut u16);
 impl Drop for Parsed{fn drop(&mut self){unsafe{LocalFree(self.0.cast());}}}
 let executable=Path::new(r"C:\Program Files\PULQVA\llama-cli.exe");
 let model=Path::new(r"C:\PULQVA data\Модель Qwen.gguf");
 let args=model_args(model);
 assert_eq!(args[1].as_os_str(),model.as_os_str());
 for (index,expected) in [(2,"--json-schema"),(3,SCHEMA),(4,"--system-prompt"),(5,SYSTEM)]{
  assert_eq!(args[index].as_os_str(),std::ffi::OsStr::new(expected));
 }
 assert!(!args.iter().any(|a|a.as_os_str()==std::ffi::OsStr::new("--json-schema-file")
  ||a.as_os_str()==std::ffi::OsStr::new("--system-prompt-file")));
 let line=command(executable,&args).unwrap();
 let mut count=0;
 let raw=unsafe{CommandLineToArgvW(line.as_ptr(),&mut count)};
 assert!(!raw.is_null(),"CommandLineToArgvW failed");
 let parsed=Parsed(raw);
 assert_eq!(count as usize,args.len()+1);
 let mut expected=vec![executable.as_os_str().to_owned()];expected.extend(args);
 for (index,value) in expected.iter().enumerate(){
  let ptr=unsafe{*parsed.0.add(index)};let mut len=0;
  while unsafe{*ptr.add(len)}!=0{len+=1;}
  let observed=unsafe{std::slice::from_raw_parts(ptr,len)};
  assert_eq!(observed,value.encode_wide().collect::<Vec<_>>(),"argv {index} changed");
 }
}
#[test]fn hex_preserves_all_bytes_and_leading_zeroes(){
 assert_eq!(hex(&[]),"");
 assert_eq!(hex(&[0,1,15,16,127,128,255]),"00010f107f80ff");
 assert_eq!(digest(b""),"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}
#[test]fn strict_stdout_boundary(){
 assert!(decode(br#"{"kind":"intent","query":"countdown video","choice_mode":"ask"}"#).is_ok());
 for s in [
  r#"log {"kind":"intent","query":"countdown","choice_mode":"ask"}"#,
  r#"{"kind":"intent","query":"countdown","choice_mode":"ask"} trailing"#,
  r#"{"kind":"intent","query":"countdown","query":"other","choice_mode":"ask"}"#,
  r#"{"kind":"intent","query":"countdown","choice_mode":"ask","command":"run"}"#,
  r#"{"kind":"intent","query":"https://example.com","choice_mode":"ask"}"#,
  r#"{"kind":"reject","reason":"semantic_authority"}"#,
 ]{assert!(decode(s.as_bytes()).is_err(),"{s}");}
 assert_eq!(digest(b"abc"),"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

const PROBE_IDS:[&str;6]=["process","model_loading","bounded_output","typed_decoding","pid_cleanup","direct_network_egress"];
fn pending_probes()->Vec<serde_json::Value>{
 PROBE_IDS.iter().map(|id|json!({"id":id,"result":"SKIP","duration_ms":null,
  "reason":"not reached / not observed"})).collect()
}
fn record_probe(probes:&mut [serde_json::Value],index:usize,result:&str,elapsed:Duration,error:Option<String>){
 probes[index]=json!({"id":PROBE_IDS[index],"result":result,
  "duration_ms":elapsed.as_millis(),"error":error});
}
fn record_failure(probes:&mut [serde_json::Value],index:usize,elapsed:Duration,error:&io::Error){
 record_probe(probes,index,"ERROR",elapsed,Some(error.to_string()));
}
fn finish_run(receipt:&mut serde_json::Value,run:io::Result<()>,cleanup:io::Result<()>,
 stdout_write:io::Result<()>,stderr_write:io::Result<()>)->io::Result<()>{
 receipt["run_error"]=json!(run.as_ref().err().map(|e|e.to_string()));
 receipt["cleanup_error"]=json!(cleanup.as_ref().err().map(|e|e.to_string()));
 receipt["stdout_write_error"]=json!(stdout_write.as_ref().err().map(|e|e.to_string()));
 receipt["stderr_write_error"]=json!(stderr_write.as_ref().err().map(|e|e.to_string()));
 run.and(cleanup).and(stdout_write).and(stderr_write)
}
#[test]fn closeout_retains_simultaneous_errors_and_original_failure(){
 let mut receipt=json!({});
 let result=finish_run(&mut receipt,Err(invalid("decode")),Err(invalid("cleanup")),
  Err(invalid("stdout disk")),Err(invalid("stderr disk")));
 assert_eq!(result.unwrap_err().to_string(),"decode");
 for (key,value) in [("run_error","decode"),("cleanup_error","cleanup"),
  ("stdout_write_error","stdout disk"),("stderr_write_error","stderr disk")]{
  assert_eq!(receipt[key],value);
 }
 for index in 0..4{
  let mut receipt=json!({});
  let mut outcomes:Vec<io::Result<()>>=(0..4).map(|i|if i==index{Err(invalid("failure"))}else{Ok(())}).collect();
  let result=finish_run(&mut receipt,outcomes.remove(0),outcomes.remove(0),outcomes.remove(0),outcomes.remove(0));
  assert!(result.is_err());
  assert_eq!(receipt.as_object().unwrap().values().filter(|v|!v.is_null()).count(),1);
 }
 let mut receipt=json!({});
 assert!(finish_run(&mut receipt,Ok(()),Ok(()),Ok(()),Ok(())).is_ok());
 assert!(receipt.as_object().unwrap().values().all(|v|v.is_null()));
}
#[test]fn failure_receipts_preserve_plan_and_cleanup_independently(){
 for index in [0,2,3]{
  let mut probes=pending_probes();
  let failure=io::Error::new(io::ErrorKind::TimedOut,"original failure");
  record_failure(&mut probes,index,Duration::from_millis(12),&failure);
  record_probe(&mut probes,4,"PASS",Duration::from_millis(3),None);
  assert_eq!(probes.len(),PROBE_IDS.len());
  for (probe,id) in probes.iter().zip(PROBE_IDS){assert_eq!(probe["id"],id);}
  assert_eq!(probes[index]["result"],"ERROR");
  assert_eq!(probes[index]["error"],"original failure");
  assert_eq!(probes[index]["duration_ms"],12);
  assert_eq!(probes[4]["result"],"PASS");
  assert_eq!(probes[5]["result"],"SKIP");
  assert!(probes[5]["duration_ms"].is_null());
 }
 let mut probes=pending_probes();
 record_failure(&mut probes,3,Duration::from_millis(12),&invalid("decode failure"));
 record_failure(&mut probes,4,Duration::from_millis(3),&invalid("cleanup failure"));
 assert_eq!(probes[3]["error"],"decode failure");
 assert_eq!(probes[4]["error"],"cleanup failure");
}

#[test]
#[ignore="explicit owner-only real Qwen AppContainer witness; never automatic CI"]
fn real_qwen_appcontainer_witness()->io::Result<()>{
 let started=Instant::now();
 let root=env_path("PULQVA_MODEL_RECEIPT_DIR")?;
 regular_path(&root,true)?;
 // A new generation only: no overwrite, no inputs from previous probes.
 let generation=root.join(format!("model-ipc-{}-{}",std::process::id(),
  SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|invalid("clock"))?.as_nanos()));
 fs::create_dir(&generation)?;
 let mut receipt=json!({"protocol":"MODEL-IPC-01/v1","test_only":true,
  "source_base":"ff2cdd1562c78b03a7754204e21d70a727b2cc01",
  "os":std::env::consts::OS,"arch":std::env::consts::ARCH,
  "prompt_sha256":digest(PROMPT.as_bytes()),"schema_sha256":digest(SCHEMA.as_bytes()),
  "system_sha256":digest(SYSTEM.as_bytes()),"deadline_seconds":180,"combined_cap":CAP,
  "overall_acceptance":false,"egress":"NOT_TESTED","probes":[]});
 let mut probes=pending_probes();
 // Setup and spawn failures belong to process; later stages explicitly take ownership.
 let mut active_probe=0usize;
 let mut active_started=Instant::now();
 let result=(||->io::Result<()>{
  let exe=env_path("PULQVA_MODEL_CLI")?;let model=env_path("PULQVA_MODEL_GGUF")?;
  let cwd=exe.parent().ok_or_else(||invalid("CLI parent missing"))?;
  regular_path(cwd,true)?;
  let cli_id=hash_file(&exe)?;let model_id=hash_file(&model)?;
  receipt["cli"]=json!(cli_id);receipt["model"]=json!(model_id);
  if cli_id!=(9052672,"c4dc5742c5dfa02b3b43f9f1839f2977c7ffc2e4ce79410f2a324943dac501d3".into())
   ||model_id!=(3013027808,"13c16f426047e2de38cd075bdade4a7bcbc8c774384876f677740cda65f8a983".into()){
   return Err(invalid("CLI/model identity mismatch"));
  }
  let mut dlls=Vec::new();
  for entry in fs::read_dir(cwd)?{
   let p=entry?.path();
   if p.extension().is_some_and(|s|s.eq_ignore_ascii_case("dll")){
    dlls.push(json!({"file":p.file_name().unwrap().to_string_lossy(),"identity":hash_file(&p)?}));
   }
  }
  receipt["adjacent_dlls"]=json!(dlls);
  receipt["dll_limit"]=json!("Inventory only; not dependency closure or authenticated DLL pins");
  let schema=generation.join("schema.json");let system=generation.join("system.txt");
  fs::write(&schema,SCHEMA)?;fs::write(&system,SYSTEM)?;
  regular_path(&schema,false)?;regular_path(&system,false)?;
  let args=model_args(&model);
  receipt["args"]=json!(args.iter().map(|s|s.to_string_lossy()).collect::<Vec<_>>());
  receipt["test_executable"]=json!(hash_file(&std::env::current_exe()?)?);
  receipt["evaluator_sha256"]=json!(digest(include_bytes!("model_ipc_witness.rs")));
  // Freeze all identities/flags and complete plan before spawn.
  receipt["probes"]=json!(probes);
  fs::write(generation.join("manifest.json"),serde_json::to_vec_pretty(&receipt)?)?;
  let (err_read,err_write)=pipe()?;
  check(unsafe{SetHandleInformation(err_read.0,HANDLE_FLAG_INHERIT,0)})?;
  let mut err=unsafe{File::from_raw_handle(err_read.0)};std::mem::forget(err_read);
  let launch=Instant::now();
  active_started=launch;
  let mut child=IsolatedChild::spawn_inner(&exe,&args,cwd,Some(err_write.0))?;
  drop(err_write);child.input.take(); // -p carries only the fixed public request; stdin EOF.
  receipt["pid"]=json!(child.pid());
  record_probe(&mut probes,0,"PASS",launch.elapsed(),None);
  active_probe=2;active_started=Instant::now();
  let mut out=Vec::new();let mut diagnostic=Vec::new();
  let run=(||->io::Result<()>{
   loop{
    if launch.elapsed()>=Duration::from_secs(180){return Err(io::Error::new(io::ErrorKind::TimedOut,"Qwen deadline"));}
    let remaining=CAP-diagnostic.len()-out.len();
    let a=drain(child.output()?,&mut out,remaining)?;
    let remaining=CAP-diagnostic.len()-out.len();
    let b=drain(&mut err,&mut diagnostic,remaining)?;
    if a&&b{
     if child.try_exit()?!=Some(0){return Err(invalid("nonzero/missing CLI exit"));}
     break;
    }
    thread::sleep(Duration::from_millis(10));
   }
   record_probe(&mut probes,2,"PASS",active_started.elapsed(),None);
   active_probe=3;active_started=Instant::now();
   decode(&out)?;
   record_probe(&mut probes,3,"PASS",active_started.elapsed(),None);
   probes[1]=json!({"id":"model_loading","result":"PASS","duration_ms":launch.elapsed().as_millis(),"basis":"pinned model, successful CLI exit and typed response"});
   Ok(())
  })();
  if let Err(e)=&run{record_failure(&mut probes,active_probe,active_started.elapsed(),e);}
  // Mark failure now, before independently attempting cleanup and evidence writes.
  active_probe=4;active_started=Instant::now();
  let close_start=Instant::now();let cleanup=child.close();
  record_probe(&mut probes,4,if cleanup.is_ok(){"PASS"}else{"ERROR"},
   close_start.elapsed(),cleanup.as_ref().err().map(|e|e.to_string()));
  // Attempt both bounded writes, even if the first fails; preserve all independent errors.
  let stdout_write=fs::write(generation.join("stdout.bin"),&out);
  let stderr_write=fs::write(generation.join("stderr.bin"),&diagnostic);
  // Operational return keeps the original run error; receipt retains cleanup/write errors too.
  finish_run(&mut receipt,run,cleanup,stdout_write,stderr_write)
 })();
 // Setup failures occur before the inner run/cleanup bookkeeping.
 if let Err(e)=&result{
  if probes[active_probe]["result"]=="SKIP"{
   record_failure(&mut probes,active_probe,active_started.elapsed(),e);
  }
 }
 receipt["probes"]=json!(probes);
 receipt["duration_ms"]=json!(started.elapsed().as_millis());
 receipt["functional_witness"]=json!(result.is_ok());
 receipt["error"]=json!(result.as_ref().err().map(|e|e.to_string()));
 receipt["limitations"]=json!("No independent egress observation; no overall acceptance. No ACL broadening, fallback or production integration. Paths assumed owner-controlled throughout hash/spawn.");
 fs::write(generation.join("matrix_receipt.json"),serde_json::to_vec_pretty(&receipt)?)?;
 result
}
