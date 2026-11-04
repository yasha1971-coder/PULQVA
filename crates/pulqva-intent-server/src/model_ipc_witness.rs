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
 let mut probes=Vec::new();
 for id in ["process","model_loading","bounded_output","typed_decoding","pid_cleanup","direct_network_egress"]{
  probes.push(json!({"id":id,"result":"SKIP","duration_ms":null,"reason":"not reached / not observed"}));
 }
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
  let args:Vec<OsString>=vec![
   "-m".into(),model.into_os_string(),"--json-schema-file".into(),schema.into_os_string(),
   "--system-prompt-file".into(),system.into_os_string(),"--single-turn".into(),
   "--simple-io".into(),"--no-display-prompt".into(),"-p".into(),PROMPT.into(),
   "-n".into(),"512".into(),"-c".into(),"4096".into(),"-t".into(),"4".into(),"--no-warmup".into()];
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
  let mut child=match IsolatedChild::spawn_inner(&exe,&args,cwd,Some(err_write.0)){
   Ok(c)=>c,Err(e)=>{probes[0]["result"]=json!("ERROR");return Err(e);}
  };
  drop(err_write);child.input.take(); // -p carries only the fixed public request; stdin EOF.
  receipt["pid"]=json!(child.pid());
  probes[0]=json!({"id":"process","result":"PASS","duration_ms":launch.elapsed().as_millis()});
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
   probes[2]=json!({"id":"bounded_output","result":"PASS","duration_ms":launch.elapsed().as_millis()});
   decode(&out)?;
   probes[3]=json!({"id":"typed_decoding","result":"PASS","duration_ms":launch.elapsed().as_millis()});
   probes[1]=json!({"id":"model_loading","result":"PASS","duration_ms":launch.elapsed().as_millis(),"basis":"pinned model, successful CLI exit and typed response"});
   Ok(())
  })();
  let close_start=Instant::now();let cleanup=child.close();
  probes[4]=json!({"id":"pid_cleanup","result":if cleanup.is_ok(){"PASS"}else{"ERROR"},
   "duration_ms":close_start.elapsed().as_millis(),"error":cleanup.as_ref().err().map(|e|e.to_string())});
  fs::write(generation.join("stdout.bin"),&out)?;
  fs::write(generation.join("stderr.bin"),&diagnostic)?;
  cleanup?;run
 })();
 receipt["probes"]=json!(probes);
 receipt["duration_ms"]=json!(started.elapsed().as_millis());
 receipt["functional_witness"]=json!(result.is_ok());
 receipt["error"]=json!(result.as_ref().err().map(|e|e.to_string()));
 receipt["limitations"]=json!("No independent egress observation; no overall acceptance. No ACL broadening, fallback or production integration. Paths assumed owner-controlled throughout hash/spawn.");
 fs::write(generation.join("matrix_receipt.json"),serde_json::to_vec_pretty(&receipt)?)?;
 result
}
