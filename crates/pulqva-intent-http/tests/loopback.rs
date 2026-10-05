use pulqva_core::{Interpretation, RejectReason};
use pulqva_intent_http::{interpret_via_loopback, LoopbackIntentEndpoint};
use std::{io::{Read,Write},net::TcpListener,thread,time::Duration};

fn serve(body:&'static str, delay_ms:u64)->u16 {
    let l=TcpListener::bind("127.0.0.1:0").unwrap(); let port=l.local_addr().unwrap().port();
    thread::spawn(move||{
        let (mut s,_)=l.accept().unwrap(); let mut b=[0u8;4096]; let _=s.read(&mut b);
        if delay_ms>0 { thread::sleep(Duration::from_millis(delay_ms)); }
        let r=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);
        let _=s.write_all(r.as_bytes());
    }); port
}
#[test] fn accepts_intent_and_reject() {
    let b=r#"{"choices":[{"message":{"content":"{\"kind\":\"intent\",\"query\":\"countdown\",\"choice_mode\":\"ask\"}"}}],"id":"fixture"}"#;
    let x=interpret_via_loopback(LoopbackIntentEndpoint::new(serve(b,0),Duration::from_secs(1)).unwrap(),"{}").unwrap();
    assert!(matches!(x,Interpretation::Intent(_)));
    let b=r#"{"choices":[{"message":{"content":"{\"kind\":\"reject\",\"reason\":\"semantic_authority\"}"}}],"id":"fixture"}"#;
    let x=interpret_via_loopback(LoopbackIntentEndpoint::new(serve(b,0),Duration::from_secs(1)).unwrap(),"{}").unwrap();
    assert_eq!(x,Interpretation::Reject(RejectReason::SemanticAuthority));
}
#[test] fn fails_closed_on_schema_bypass_and_timeout() {
    for b in [
      r#"{"choices":[{"message":{"content":"{\"kind\":\"reject\",\"reason\":\"semantic_authority\",\"url\":\"https://example.com\"}"}}]}"#,
      r#"{"choices":[{"message":{"content":"not json"}}]}"#,
      r#"{"choices":[],"id":"no-choice"}"#
    ] {
      assert!(interpret_via_loopback(LoopbackIntentEndpoint::new(serve(b,0),Duration::from_secs(1)).unwrap(),"{}").is_err());
    }
    assert!(interpret_via_loopback(LoopbackIntentEndpoint::new(serve(r#"{"choices":[{"message":{"content":"x"}}]}"#,300),Duration::from_millis(20)).unwrap(),"{}").is_err());
}
