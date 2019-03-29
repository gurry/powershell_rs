use std::{fmt, process, str};
use failure::Fail;

pub use std::process::{
    Stdio,
    ExitStatus,
    Output,
    ChildStdin as Stdin,
    ChildStdout as Stdout,
    ChildStderr as Stderr,
};


// TODO: a lot of this stuff needs rethought
// For example which of the existing types in
// std::process we should re-export and which
// ones we should wrap in our own types

const POWERSHELL_EXE: &str = "powershell.exe";

pub struct PsCommand {
    command: process::Command,
}

impl PsCommand {
    pub fn new<C: AsRef<str>>(command_str: C) -> Self {
        Self { command: Self::create_command(command_str.as_ref()) }
    }

    fn create_command(command_str: &str) -> process::Command {
        let mut command = process::Command::new(POWERSHELL_EXE);
        command.arg("-NoProfile").arg("-NonInteractive").arg("-NoLogo").arg("-ExecutionPolicy").arg("Bypass").arg("-Command");

        for part in command_str.split_whitespace() {
            // TODO: here ensure that none of the 'part's are
            // match or is in conflit with the standard args
            // like "-NoProfile" we've specified above.
            // If any of them is, then return failure 
            command.arg(part);
        }

        command
    }

    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.command.stdin(cfg);
        self
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.command.stdout(cfg);
        self
    }

    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.command.stderr(cfg);
        self
    }

    pub fn spawn(&mut self) -> Result<PsProcess, PsError> {
        let child = self.command.spawn()
            .map_err(|e| PsError{ msg: format!("Failed to spawn: {}", e) })?;
        Ok(PsProcess(child))
    }

    pub fn output(&mut self) -> Result<Output, PsError> {
        self.command.output()
            .map_err(|e| PsError{ msg: format!("Failed to spawn: {}", e) })
    }

    pub fn status(&mut self) -> Result<ExitStatus, PsError> {
        self.command.status()
            .map_err(|e| PsError{ msg: format!("Failed to spawn: {}", e) })
    }
}

pub struct PsProcess(process::Child);

impl PsProcess {
    pub fn stdin(self) -> Option<Stdin> {
        self.0.stdin
    }

    pub fn stdout(self) -> Option<Stdout> {
        self.0.stdout
    }

    pub fn stderr(self) -> Option<Stderr> {
        self.0.stderr
    }

    pub fn kill(&mut self) -> Result<(), PsError> {
        self.0.kill()
            .map_err(|e| PsError { msg: e.to_string() })
    }

    pub fn id(&self) -> u32 {
        self.0.id()
    }

    pub fn wait(&mut self) -> Result<ExitStatus, PsError> {
         self.0.wait()
            .map_err(|e| PsError { msg: e.to_string() })
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, PsError> {
         self.0.try_wait()
            .map_err(|e| PsError { msg: e.to_string() })
    }

    pub fn wait_with_output(self) -> Result<Output, PsError> {
         self.0.wait_with_output()
            .map_err(|e| PsError { msg: e.to_string() })
    }
}

pub fn ps_version() -> Result<PsVersion, PsError> {
    let output = PsCommand::new("$PSVersionTable.PSVersion.ToString()")
        .output()
        .map_err(|e| PsError { msg: format!("Failed to spawn powershell process to read from the version table: {}", e) })?;

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
        let version_str = version_str.trim();
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