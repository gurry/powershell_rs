use powershell_rs::Ps;

fn main() {
    println!("Running 'ls'...");
    match Ps::execute("ls") {
        Ok(output) => {
            println!("Exit code: {:?}", output.exit_code());
            println!("Stdout output:");
            println!("{}", output.stdout());
        },
        Err(e) => println!("ls failed with error: {}", e),
    }
}