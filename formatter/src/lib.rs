extern crate git2;
extern crate serde;
extern crate toml;

use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;

use git2::{Repository, Status};
use serde::Deserialize;

const CONFIG_NAME: &'static str = "GitFormat.toml";

const FILE_TEMPLATE: &'static str = "{{STAGED_FILE}}";

type FormatterDirectory = HashMap<String, FormatterOption>;

pub struct Formatter {
    formatters: FormatterDirectory,
    repo: Repository,
}

#[derive(Debug, Deserialize)]
struct FormatterOption {
    /// Template for the formatting command.
    command: Option<String>,
    /// All file extensions that apply to the command.
    extensions: Option<Vec<String>>,
}

impl Formatter {
    /// This method finds config files locating in the root directory of the
    /// repository. This allows `git fmt` to work from anywhere within a repository.
    /// This method does not work in bare repositories.
    pub fn from_root_config() -> Result<Self, Box<dyn Error>> {
        let repo = Repository::open(".")?;
        let workdir = repo.workdir().unwrap_or(Path::new("."));
        let config_content = read_to_string(workdir.join(CONFIG_NAME))?;
        Self::new(&config_content, repo)
    }

    /// Formats all the files currently checked into the index with a valid
    /// formatter. If a file is contested (both in the index and in workdir),
    /// this method will not format that file.
    pub fn format_index(&self) -> Result<(), Box<dyn Error>> {
        // StatusOptions defaults to including both workspace and index files. If the
        // same file exists in both the index and the workspace, the file's status
        // will include both.
        let statuses = self.repo.statuses(None)?;

        for status in statuses.iter() {
            match status.status() {
                Status::INDEX_MODIFIED | Status::INDEX_NEW => {
                    if let Some(pathname) = status.path() {
                        self.execute_formatter(&pathname);
                    }
                }
                _ => (),
            };
        }
        return Ok(());
    }

    // Private
    fn new(config: &str, repo: Repository) -> Result<Self, Box<dyn Error>> {
        let mut formatters: FormatterDirectory = toml::from_str(config)?;
        formatters.retain(|_, v| v.command.is_some() && v.extensions.is_some());
        Ok(Self { formatters, repo })
    }

    fn get_command(&self, pathname: &Path) -> Option<&String> {
        let ext = pathname.extension();
        if let Some(e) = ext {
            let extension_name = e.to_os_string().into_string().unwrap();
            for (_, formatter) in self.formatters.iter() {
                let extensions = formatter.extensions.as_ref().unwrap();
                if extensions.contains(&extension_name) {
                    return formatter.command.as_ref();
                }
            }
        }
        None
    }

    fn execute_formatter<S: AsRef<OsStr> + Sized>(&self, pathname: &S) {
        let path = Path::new(pathname);
        let cmd = self.get_command(&path);
        if let Some(c) = cmd {
            let parsed_cmd = c.replace(FILE_TEMPLATE, path.to_str().unwrap());
            let mut c: Vec<&str> = parsed_cmd.split(' ').collect();

            Command::new(c.remove(0))
                .args(c)
                .spawn()
                .expect("failed to execute formatting command");
        }
    }
}
