use std::{collections::HashSet, env, fs, path::PathBuf, process};

use cooklang::{Converter, CooklangParser, Extensions, Recipe};
use itertools::{Either, Itertools};
use walkdir::WalkDir;

const MIN_LEVENSHTEIN_DISTANCE: usize = 2;
const LEVENSHTEIN_IGNORE: [(&str, &str); 3] =
    [("salt", "kale"), ("rice", "lime"), ("mint", "milk")];

#[derive(Debug)]
struct LintResult {
    parse_failures: Vec<String>,
    ingredients_no_aisle: Vec<String>,
    duplicate_ingredients: Vec<(String, String)>,
}

impl LintResult {
    fn summarize(&self) {
        if self.parse_failures.len() > 0 {
            println!("Parse failures:");

            for parse_failure in &self.parse_failures {
                println!("{parse_failure}");
            }
        } else {
            println!("All recipes parsed successfully");
        }
        println!("");

        if self.ingredients_no_aisle.len() > 0 {
            println!("Ingredients missing an aisle:");

            for parse_failure in &self.ingredients_no_aisle {
                println!("{parse_failure}");
            }
        } else {
            println!("All ingredients have an aisle");
        }
        println!("");

        if self.duplicate_ingredients.len() > 0 {
            println!("Duplicate ingredients found:");

            for (i0, i1) in &self.duplicate_ingredients {
                println!("{i0} {i1}");
            }
        } else {
            println!("No duplicate ingredients found");
        }
        println!("");
    }

    fn is_success(&self) -> bool {
        self.duplicate_ingredients.is_empty()
            && self.ingredients_no_aisle.is_empty()
            && self.parse_failures.is_empty()
    }
}

fn main() {
    let parser = CooklangParser::new(Extensions::all(), Converter::default());

    let recipes_dir = get_recipes_dir();

    let recipes = get_all_recipes(recipes_dir.clone());

    let (parsed_recipes, parse_failures) = parse_recipes(&recipes, &parser);

    let ingredients: HashSet<String> = parsed_recipes
        .iter()
        .flat_map(|r| r.ingredients.iter().map(|i| i.name.clone()))
        .collect();

    let aisle_config_path = recipes_dir.join("config").join("aisle.conf");
    let aisle_config_file = fs::read_to_string(aisle_config_path).unwrap();
    let aisle_config = cooklang::aisle::parse(&aisle_config_file).unwrap();
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
}

fn get_all_recipes(dir: PathBuf) -> Vec<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().is_some()
                && e.path().extension().unwrap() == "cook"
        })
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

fn get_recipes_dir() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = PathBuf::from(manifest_dir);
    manifest_dir.parent().unwrap().join("recipes")
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
