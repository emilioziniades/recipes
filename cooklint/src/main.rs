pub mod cli;
pub mod lint;

use std::{collections::HashSet, ffi::OsStr, fs, path::PathBuf, process};

use anyhow::Context;
use anyhow::anyhow;
use clap::Parser;
use cooklang::{Converter, CooklangParser, Extensions, Recipe};
use itertools::{Either, Itertools};
use walkdir::WalkDir;

use crate::lint::LintResult;

const MIN_LEVENSHTEIN_DISTANCE: usize = 2;
const LEVENSHTEIN_IGNORE: [(&str, &str); 5] = [
    ("salt", "kale"),
    ("rice", "lime"),
    ("mint", "milk"),
    ("pepper", "peppers"),
    ("broccoli", "broccolini"),
];

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let parser = CooklangParser::new(Extensions::all(), Converter::default());

    let recipes_dir = PathBuf::from(args.dir);

    let recipes = get_all_recipes(recipes_dir.clone());

    if recipes.len() == 0 {
        return Err(anyhow!("No recipes found in {:?}", recipes_dir));
    }

    let (parsed_recipes, parse_failures) = parse_recipes(&recipes, &parser);

    let ingredients: HashSet<String> = parsed_recipes
        .iter()
        .flat_map(|r| r.ingredients.iter().map(|i| i.name.clone()))
        .collect();

    let aisle_config_path = recipes_dir.join("config").join("aisle.conf");
    let aisle_config_file =
        fs::read_to_string(aisle_config_path).context("reading config/aisle.conf")?;
    let aisle_config =
        cooklang::aisle::parse(&aisle_config_file).context("parsing config/aisle.conf")?;

    let aisle_ingredients: HashSet<String> = aisle_config
        .categories
        .iter()
        .flat_map(|c| c.ingredients.iter().flat_map(|i| i.names.clone()))
        .map(str::to_string)
        .collect();

    let ingredients_no_aisle: Vec<String> = ingredients
        .iter()
        .filter(|i| !aisle_ingredients.contains(*i))
        .map(Clone::clone)
        .collect();

    let duplicate_ingredients: Vec<(String, String)> =
        find_duplicate_ingredients(&ingredients, MIN_LEVENSHTEIN_DISTANCE);

    let lint_result = LintResult {
        parse_failures,
        ingredients_no_aisle,
        duplicate_ingredients,
    };

    lint_result.summarize();

    if !lint_result.is_success() {
        process::exit(1);
    }

    Ok(())
}

fn get_all_recipes(dir: PathBuf) -> Vec<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension() == Some(OsStr::new("cook")))
        .map(|e| e.path().to_path_buf())
        .map(|pb| fs::read_to_string(pb).unwrap())
        .collect()
}

fn parse_recipes(recipes: &Vec<String>, parser: &CooklangParser) -> (Vec<Recipe>, Vec<String>) {
    recipes
        .iter()
        .map(|r| parser.parse(r).into_result())
        .partition_map(|r| match r {
            Ok((recipe, _)) => Either::Left(recipe),
            Err(err) => Either::Right(err.to_string()),
        })
}

fn find_duplicate_ingredients(
    ingredients: &HashSet<String>,
    min_distance: usize,
) -> Vec<(String, String)> {
    let false_positives: HashSet<(&str, &str)> = LEVENSHTEIN_IGNORE.into_iter().collect();
    ingredients
        .iter()
        .combinations(2)
        .into_iter()
        .map(|is| (is[0].clone(), is[1].clone()))
        .filter(|(i0, i1)| levenshtein::levenshtein(i0, i1) <= min_distance)
        .filter(|(i0, i1)| {
            !false_positives.contains(&(i0, i1)) && !false_positives.contains(&(i1, i0))
        })
        .collect()
}
