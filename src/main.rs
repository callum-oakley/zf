#![warn(clippy::pedantic)]

use std::{env, fs, iter, path::Path, process::Command};

use anyhow::bail;
use regex::{Captures, Regex};

struct Script<'a> {
    name: &'a str,
    parameters: Vec<&'a str>,
    body: &'a str,
}

impl<'a> Script<'a> {
    fn from(captures: &Captures<'a>) -> Self {
        let signature = captures.get(1).unwrap().as_str();
        let body = captures.get(2).unwrap().as_str().trim_end();

        let mut words = signature.split_whitespace();
        let name = words.next().unwrap();
        let arguments = words.collect::<Vec<_>>();

        Script {
            name,
            parameters: arguments,
            body,
        }
    }

    fn run(&self, arguments: &[String]) -> anyhow::Result<()> {
        let mut cmd = Command::new(env::var("SHELL")?);
        cmd.args(["-c", self.body, self.name]);
        for (parameter, argument) in iter::zip(&self.parameters, arguments) {
            cmd.env(parameter, argument);
        }
        cmd.args(&arguments[self.parameters.len()..]);

        let status = cmd.status()?;
        if !status.success() {
            bail!(status);
        }

        Ok(())
    }

    fn print(&self) {
        let indentation = self.body.chars().take_while(|&c| c == ' ').count();
        for line in self.body.lines() {
            eprintln!("> {}", line.split_at(indentation.min(line.len())).1);
        }
    }
}

fn parse(scriptfile: &str) -> anyhow::Result<Vec<Script>> {
    let comment = r"#.*\n| *\n";
    let script_re = Regex::new(&format!(r"(?:{comment})*([^# ].*\n)((?: .*\n|\n)*)"))?;
    let scriptfile_re = Regex::new(&format!(r"^({script_re})*({comment})*$"))?;

    if !scriptfile_re.is_match(scriptfile) {
        bail!("malformed scriptfile");
    }

    Ok(script_re
        .captures_iter(scriptfile)
        .map(|c| Script::from(&c))
        .collect())
}

fn main() -> anyhow::Result<()> {
    if !Path::new("scriptfile").try_exists()? {
        bail!("no scriptfile in current directory");
    };

    let scriptfile = fs::read_to_string("scriptfile")?;
    let scripts = parse(&scriptfile)?;

    let args = env::args().collect::<Vec<_>>();
    let args = &args[1..];
    if args.is_empty() {
        eprint!("{scriptfile}");
        return Ok(());
    }

    let name = &args[0];
    let arguments = &args[1..];
    let arity = arguments.len();

    let Some(script) = scripts
        .iter()
        .find(|r| r.name == name && r.parameters.len() <= arity)
    else {
        bail!("no scripts with name {name} and {arity} parameters");
    };

    script.print();
    script.run(arguments)
}
