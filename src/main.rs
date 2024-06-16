use std::{env, fs, iter, path::Path, process::Command};

use anyhow::bail;
use regex::{Captures, Regex};

#[derive(Debug)]
struct Recipe<'a> {
    name: &'a str,
    arguments: Vec<&'a str>,
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
            arguments,
            body: method,
        }
    }

    fn print(&self) {
        let indentation = self.body.chars().take_while(|&c| c == ' ').count();
        for line in self.body.lines() {
            eprintln!("> {}", line.split_at(indentation).1);
        }
    }

    fn run(&self, values: &[String]) -> anyhow::Result<()> {
        let mut cmd = Command::new("/bin/sh");
        cmd.args(["-c", self.body]);
        for (ingredient, value) in iter::zip(&self.arguments, values) {
            cmd.env(ingredient, value);
        }

        let status = cmd.status()?;
        if !status.success() {
            bail!(status);
        }

        Ok(())
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
    let values = &args[1..];
    let arity = values.len();

    let Some(recipe) = recipes
        .iter()
        .find(|r| r.name == name && r.arguments.len() == arity)
    else {
        bail!("no recipes with name {name} and {arity} arguments");
    };

    recipe.print();
    recipe.run(values)
}
