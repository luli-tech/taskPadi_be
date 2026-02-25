// async-nats test
#[tokio::main]
async fn main() {
    // The NATS credential file MUST contain both headers exactly as shown here so the parser can find the JWT and the user NKEY Seed.
    let creds_str = r#"-----BEGIN NATS USER JWT-----
eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJPWlY3STRPTFRaN1pZTk80UVRCU1VSQVJXNUgzR09HQ1hXREJGUkNUR0hVUUVJUU1BRDNRIiwiaWF0IjoxNzcxNzk4MDc5LCJpc3MiOiJBRFlYWjY3WDVDWEY2M0xDSlBBVUZNSEYzNjcyR0ZGRkFYSEVBR0FGU1IzNVg3STZMSjVWUVBaUiIsIm5hbWUiOiJDTEkiLCJzdWIiOiJVQVBVTERVSklNS1dPR0VSNkg2RTcyUlUzN0VOUkxZRTY1NkVZUVRZMldIS0dEVlpTNkhNNEVGVCIsIm5hdHMiOnsicHViIjp7fSwic3ViIjp7fSwic3VicyI6LTEsImRhdGEiOi0xLCJwYXlsb2FkIjotMSwiaXNzdWVyX2FjY291bnQiOiJBQlpPVEpXU05DQU1RNllVUDRMRE40VEhIRVBLRlpRREFWVUhXV1U0QVFGVUg3WjZVTzZFUkxNVyIsInR5cGUiOiJ1c2VyIiwiY29kZSI6Mn19.YWsYxSnKRS8St4pFeupcwUs6Bii4X3hj40BKgHoRX5BnosLWjPPAXfAbshRPyyRAPXvSSVor6hBJ1MbhBgyzCw
------END NATS USER JWT------

************************* IMPORTANT *************************
NKEY Seed printed below can be used to sign and prove identity.
NKEYs are sensitive and should be treated as secrets.

-----BEGIN USER NKEY SEED-----
SUADYN3HVZY4CEGZAIMARZBF6XHSZASLGJPYLSDW4NXSFBPHNF4RIW3XJU
------END USER NKEY SEED------
"#;
    
    // Windows file
    let path_win = std::env::temp_dir().join("nats_win.creds");
    std::fs::write(&path_win, creds_str.replace("\n", "\r\n")).unwrap();
    let res_win = async_nats::ConnectOptions::with_credentials_file(&path_win).await;
    println!("Windows (CRLF) parsed properly? {:?}", res_win.is_ok());

    // Unix file
    let path_unix = std::env::temp_dir().join("nats_unix.creds");
    std::fs::write(&path_unix, creds_str.replace("\r\n", "\n")).unwrap();
    let res_unix = async_nats::ConnectOptions::with_credentials_file(&path_unix).await;
    println!("Unix (LF) parsed properly? {:?}", res_unix.is_ok());
    
    if res_unix.is_ok() {
        // Let's actually test connecting to the cloud to prove it works
        let final_result = res_unix.unwrap().connect("tls://connect.ngs.global:4222").await;
        println!("Cloud connection result: {:?}", final_result.is_ok());
    }
}
