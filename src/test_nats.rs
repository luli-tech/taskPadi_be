// async-nats test
#[tokio::main]
async fn main() {
    let creds_str = "-----BEGIN NATS USER JWT-----\neyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJPWlY3STRPTFRaN1pZTk80UVRCU1VSQVJXNUgzR09HQ1hXREJGUkNUR0hVUUVJUU1BRDNRIiwiaWF0IjoxNzcxNzk4MDc5LCJpc3MiOiJBRFlYWjY3WDVDWEY2M0xDSlBBVUZNSEYzNjcyR0ZGRkFYSEVBR0FGU1IzNVg3STZMSjVWUVBaUiIsIm5hbWUiOiJDTEkiLCJzdWIiOiJVQVBVTERVSklNS1dPR0VSNkg2RTcyUlUzN0VOUkxZRTY1NkVZUVRZMldIS0dEVlpTNkhNNEVGVCIsIm5hdHMiOnsicHViIjp7fSwic3ViIjp7fSwic3VicyI6LTEsImRhdGEiOi0xLCJwYXlsb2FkIjotMSwiaXNzdWVyX2FjY291bnQiOiJBQlpPVEpXU05DQU1RNllVUDRMRE40VEhIRVBLRlpRREFWVUhXV1U0QVFGVUg3WjZVTzZFUkxNVyIsInR5cGUiOiJ1c2VyIiwiY29kZSI6Mn19.YWsYxSnKRS8St4pFeupcwUs6Bii4X3hj40BKgHoRX5BnosLWjPPAXfAbshRPyyRAPXvSSVor6hBJ1MbhBgyzCw\n------END NATS USER JWT------\nSUADYN3HVZY4CEGZAIMARZBF6XHSZASLGJPYLSDW4NXSFBPHNF4RIW3XJU";
    
    // Windows file
    let path_win = std::env::temp_dir().join("nats_win.creds");
    std::fs::write(&path_win, creds_str.replace("\n", "\r\n")).unwrap();
    let res_win = async_nats::ConnectOptions::new().credentials(path_win.to_str().unwrap());
    println!("Windows (CRLF): {:?}", res_win.is_ok());

    // Unix file
    let path_unix = std::env::temp_dir().join("nats_unix.creds");
    std::fs::write(&path_unix, creds_str.replace("\r\n", "\n")).unwrap();
    let res_unix = async_nats::ConnectOptions::new().credentials(path_unix.to_str().unwrap());
    println!("Unix (LF): {:?}", res_unix.is_ok());
}
