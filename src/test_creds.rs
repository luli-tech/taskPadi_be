use std::env;

fn main() {
    let creds = "-----BEGIN NATS USER JWT-----\neyJ0eXAiOiJ....\n------END NATS USER JWT------\nSUADYN3HVZY...";
    let parsed_creds = creds.replace("\\n", " ").replace("\\r", " ").trim_matches('"').to_string();
    let mut clean_creds = parsed_creds.clone();
    if let (Some(jwt_start), Some(seed_start)) = (parsed_creds.find("eyJ"), parsed_creds.find("SU")) {
        let jwt: String = parsed_creds[jwt_start..].chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '.' || *c == '_').collect();
        let seed: String = parsed_creds[seed_start..].chars().take_while(|c| c.is_alphanumeric()).collect();
        
        println!("JWT: '{}'", jwt);
        println!("SEED: '{}'", seed);
        
        if !jwt.is_empty() && !seed.is_empty() {
            clean_creds = format!(
                "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n************************* IMPORTANT *************************\nNKEY Seed printed below can be used to sign and prove identity.\nNKEYs are sensitive and should be treated as secrets.\n\n-----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n",
                jwt, seed
            );
            println!("CLEAN CREDS:\n{}", clean_creds);
        }
    }
}
