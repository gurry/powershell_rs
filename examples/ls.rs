use powershell_rs::Ps;

fn main() {
    match Ps::version() {
        Ok(version) => println!("Powershell version {}", version),
        Err(e) => println!("Failed to get version. Error: {}", e),
    };

    println!("Running 'ls'...");
    match Ps::execute("ls") {
        Ok(output) => {
            println!("Exit code: {:?}", output.exit_code());
            println!("Stdout output:");
            println!("{}", output.stdout());
        },
        Err(e) => println!("Failed with error: {}", e),
    }
}