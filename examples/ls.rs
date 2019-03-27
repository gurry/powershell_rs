use powershell_rs::{PsCommand, ps_version};

fn main() {
    match ps_version() {
        Ok(version) => println!("Powershell version {}", version),
        Err(e) => println!("Failed to get version. Error: {}", e),
    };

    println!("Running 'ls'...");
    match PsCommand::new("ls").output() {
        Ok(output) => {
            if let Some(code) = output.status.code() {
                println!("Exit code: {:?}", code)
            }
            
            println!("Stdout output:");
            println!("{}", String::from_utf8_lossy(output.stdout.as_slice()));
        },
        Err(e) => println!("Failed with error: {}", e),
    }
}