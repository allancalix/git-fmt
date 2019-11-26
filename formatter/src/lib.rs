extern crate git2;
extern crate log;

use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use git2::{Repository, Status};
use log::warn;

const FILE_TEMPLATE: &'static str = "{{STAGED_FILE}}";

const OPTION_KEY: &'static str = "fmt";

type FormatterDirectory = HashMap<String, FormatterOption>;

pub struct Formatter {
    formatters: FormatterDirectory,
    repo: Repository,
}

#[derive(Debug)]
struct FormatterOption {
    /// Template for the formatting command.
    command: Option<String>,
    /// All file extensions that apply to the command.
    extensions: Option<Vec<String>>,
}

type Opt = HashMap<String, String>;

/// Collects fmt options into map keyed by the language.
/// language -> ...options
fn collect_formatters(config: ::git2::Config) -> HashMap<String, Opt> {
    let mut formatters = HashMap::new();

    for entry in &config.entries(None).unwrap() {
        let entry = entry.unwrap();
        let path: Vec<&str> = entry.name().unwrap().split(".").collect();

        if let Some(name) = path.get(0) {
            if name == &OPTION_KEY {
                if path.len() != 3 {
                    warn!(
                        "Option {} does not match format (fmt.<language>.<option>).",
                        entry.name().unwrap()
                    );
                    continue;
                }

                let lang = path.get(1).unwrap();
                let opt = path.get(2).unwrap();

                let item = formatters
                    .entry(lang.to_string())
                    .or_insert_with(|| HashMap::new());

                item.insert(opt.to_string(), entry.value().unwrap().to_string());
            }
        }
    }
    formatters
}

impl Formatter {
    /// Initialize a formatter using the git config values belonging to the local
    /// workspace. Final results will depend on values in parent configs with local
    /// values taking precedent. See git-config ancestry rules for more information.
    pub fn from_local_workspace() -> Result<Self, Box<dyn Error>> {
        let repo = Repository::open(".")?;
        let config = repo.config()?;
        let opts = collect_formatters(config);

        let mut formatters = HashMap::new();
        for (lang, opt) in &opts {
            let cmd = opt.get("command").unwrap();
            let ext: Vec<&str> = opt.get("extensions").unwrap().split(',').collect();
            let parsed: Vec<String> = ext.into_iter().map(|s| s.trim().to_string()).collect();

            formatters.insert(
                lang.to_string(),
                FormatterOption {
                    command: Some(cmd.to_string()),
                    extensions: Some(parsed),
                },
            );
        }

        Ok(Self { formatters, repo })
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
