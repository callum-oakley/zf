use std::{env, fs, iter, path::Path, process::Command};

use anyhow::bail;
use regex::{Captures, Regex};

#[derive(Debug)]
struct Recipe<'a> {
    name: &'a str,
    parameters: Vec<&'a str>,
    body: &'a str,
}

impl<'a> Recipe<'a> {
    fn from(captures: Captures<'a>) -> Self {
        let signature = captures.get(1).unwrap().as_str();
        let method = captures.get(2).unwrap().as_str();

        let mut words = signature.split_whitespace();
        let name = words.next().unwrap();
        let arguments = words.collect::<Vec<_>>();

        Recipe {
            name,
            parameters: arguments,
            body: method,
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
            eprintln!("> {}", line.split_at(indentation).1);
        }
    }
}

fn parse(cookbook: &str) -> anyhow::Result<Vec<Recipe>> {
    let comment = r"#.*\n| *\n";
    let recipe_re = Regex::new(&format!(r"(?:{comment})*([^# ].*\n)((?: .*\n)*)"))?;
    let cookbook_re = Regex::new(&format!(r"^({recipe_re})*({comment})*$"))?;

    if !cookbook_re.is_match(cookbook) {
        bail!("malformed cookbook");
    }

    Ok(recipe_re
        .captures_iter(cookbook)
        .map(Recipe::from)
        .collect())
}

fn main() -> anyhow::Result<()> {
    if !Path::new("cookbook").try_exists()? {
        bail!("no cookbook in current directory");
    };

    let cookbook = fs::read_to_string("cookbook")?;
    let recipes = parse(&cookbook)?;

    let args = env::args().collect::<Vec<_>>();
    let args = &args[1..];
    if args.is_empty() {
        eprint!("{cookbook}");
        return Ok(());
    }

    let name = &args[0];
    let arguments = &args[1..];
    let arity = arguments.len();

    let Some(recipe) = recipes
        .iter()
        .find(|r| r.name == name && r.parameters.len() <= arity)
    else {
        bail!("no recipes with name {name} and {arity} parameters");
    };

    recipe.print();
    recipe.run(arguments)
}
