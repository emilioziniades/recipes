#[derive(Debug)]
pub struct LintResult {
    pub parse_failures: Vec<String>,
    pub ingredients_no_aisle: Vec<String>,
    pub duplicate_ingredients: Vec<(String, String)>,
}

impl LintResult {
    pub fn summarize(&self) {
        if self.parse_failures.len() > 0 {
            println!("FAIL: Parse failures");

            for parse_failure in &self.parse_failures {
                println!("\t{parse_failure}");
            }
        } else {
            println!("PASS: All recipes parsed successfully");
        }

        if self.ingredients_no_aisle.len() > 0 {
            println!("FAIL: Ingredients missing an aisle");

            for parse_failure in &self.ingredients_no_aisle {
                println!("\t{parse_failure}");
            }
        } else {
            println!("PASS: All ingredients have an aisle");
        }

        if self.duplicate_ingredients.len() > 0 {
            println!("FAIL: Duplicate ingredients found");

            for (i0, i1) in &self.duplicate_ingredients {
                println!("\t{i0} {i1}");
            }
        } else {
            println!("PASS: No duplicate ingredients found");
        }
    }

    pub fn is_success(&self) -> bool {
        self.duplicate_ingredients.is_empty()
            && self.ingredients_no_aisle.is_empty()
            && self.parse_failures.is_empty()
    }
}
