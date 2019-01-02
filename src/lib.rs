use std::{fmt, process, str};
use failure::Fail;

const POWERSHELL_EXE: &str = "powershell.exe";
pub struct Ps;

impl Ps {
    pub fn execute(command: &str) -> Result<PsOutput, PsError> {
        let output = Self::invoke_ps(command)?;
        Ok(PsOutput{ inner: output })
    }

    pub fn version() -> Result<PsVersion, PsError> {
        let output = Self::invoke_ps("$PSVersionTable.PSVersion.ToString()")?;

        if !output.status.success() {
            let code_str = if output.status.code().is_some() {
                output.status.code().unwrap().to_string()
            } else {
                "<unknown>".to_owned()
            };
            return Err(PsError { msg: format!("Reading from version table failed with exit code {}", code_str) });
        }

        let version = to_string(&output.stdout).parse::<PsVersion>()?;
        Ok(version)
    }

    fn invoke_ps(command: &str) -> Result<process::Output, PsError> {
        let mut ps_process = process::Command::new(POWERSHELL_EXE);
        let ps_process = ps_process
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-NoLogo")
            .arg("-ExecutionPolicy ")
            .arg("Bypass")
            .arg("-Command");

        for part in command.split_whitespace() {
            ps_process.arg(part);
        }
        
        let output = ps_process.output()
            .map_err(|e| PsError { msg: format!("Error while spawning {}: {}", POWERSHELL_EXE, e) })?;
        Ok(output)
    }
}

// TODO: The current shape of PsOutput may not be best
// we need to think of how we'll handle the case when a 
// command spits out a large stream of data. A String
// type won't do in that case. Fix this.
#[derive(Debug)]
pub struct PsOutput {
    inner: process::Output,
}

impl PsOutput {
    pub fn stdout(&self) -> String {
        to_string(&self.inner.stdout)
    }

    pub fn stderr(&self) -> String {
        to_string(&self.inner.stderr)
    }
    
    pub fn exit_code(&self) -> Option<i32> {
        self.inner.status.code()
    }
}

// TODO: We need to do proper design of error types. Just this one type is not enough
#[derive(Debug, Fail)]
pub struct PsError {
    pub msg: String,
}

impl fmt::Display for PsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[derive(Debug)]
pub struct PsVersion {
    pub major: u32,
    pub minor: u32,
    pub build: i32,
    pub revision: i32
}

impl str::FromStr for PsVersion {
    type Err = PsError;
    fn from_str(version_str: &str) -> Result<Self, Self::Err> {
        // TODO: Optimize this. Avoid allocations if we can.
        let parts = version_str.split('.').collect::<Vec<_>>();
        let error = || PsError { msg: format!("Cannot parse '{}' into PowerShell version", version_str) };

        if parts.len() != 4 {
            return Err(error());
        }

        let major = parts[0].parse::<u32>().map_err(|_| error())?;
        let minor = parts[1].parse::<u32>().map_err(|_| error())?;
        let build = parts[2].parse::<i32>().map_err(|_| error())?;
        let revision = parts[3].parse::<i32>().map_err(|_| error())?;

        Ok(Self { major, minor, build, revision })
    }
}

impl fmt::Display for PsVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.major, self.minor, self.build, self.revision)
    }
}

fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
