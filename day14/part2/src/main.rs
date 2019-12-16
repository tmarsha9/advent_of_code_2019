use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ChemicalUnit {
    id: String,
    amount: u64
}

#[derive(Debug, Clone)]
struct Recipe {
    product: ChemicalUnit,
    ingredients: Vec<ChemicalUnit>
}

struct Reactor {
    recipes: HashMap<String, Recipe>,
    quantities: HashMap<String, u64>,
    ore_consumed: u64,
}

impl Reactor {
    fn new() -> Self {
        Reactor {
            recipes: HashMap::new(),
            quantities: HashMap::new(),
            ore_consumed: 0u64
        }
    }

    fn add_recipe(&mut self, r: Recipe) {
        self.quantities.insert(r.product.id.clone(), 0);
        self.recipes.insert(r.product.id.clone(), r);
    }

    fn do_recipe_for_product(&mut self, product_id: &String, num_times: u64) {
        // doing roundabout method because can't borrow self as mutable twice, so no recursion
        // recursion was original plan
        loop {
            // ensure enough ingredients
            let mut not_enough_of_this_ingredient = None;
            let mut have = 0u64;
            let mut need = 0u64;
            let recipe = self.recipes.get(product_id).unwrap();
            for ingredient in recipe.ingredients.iter() {
                if ingredient.id == "ORE" {
                    // always able to get ore
                    continue;
                } else {
                    let current_amount = self.quantities.get(&ingredient.id).unwrap();
                    if current_amount < &(ingredient.amount*num_times) {
                        not_enough_of_this_ingredient = Some(ingredient.id.clone());
                        have = *current_amount;
                        need = ingredient.amount*num_times;
                        break;
                    }
                }
            }

            if let Some(missing_ingredient_id) = not_enough_of_this_ingredient {
                let r = self.recipes.get(&missing_ingredient_id).unwrap().product.amount;
                let num_times = if (need - have) % r == 0 {
                    (need-have)/r
                } else {
                    ((need-have)/r) + 1
                };
                self.do_recipe_for_product(&missing_ingredient_id, num_times);
            } else {
                break;
            }
        }

        // all dependent ingredients present
        // update all quantities
        let recipe = self.recipes.get(product_id).unwrap();
        // deduct ingredients
        for ingredient in recipe.ingredients.iter() {
            if ingredient.id == "ORE" {
                self.ore_consumed += ingredient.amount*num_times;
            } else {
                *self.quantities.get_mut(&ingredient.id).unwrap() -= ingredient.amount*num_times;
            }
        }
        // add product
        *self.quantities.get_mut(product_id).unwrap() += recipe.product.amount*num_times;
    }

    fn reset(&mut self) {
        self.ore_consumed = 0;
        self.quantities = HashMap::new();
        for (product_id, _) in self.recipes.iter() {
            if product_id != "ORE" {
                self.quantities.insert(product_id.clone(), 0);
            }
        }
    }
}

fn main() {
    let f = File::open("../input.txt").expect("couldn't open input");
    let f = BufReader::new(f);

    let mut reactor = Reactor::new();
    for line in f.lines() {
        let line = line.unwrap();
        let line: Vec<&str> = line.split("=>").map(|s| s.trim()).collect();
        let product_unparsed: Vec<&str> = line[1].split(" ").collect();
        let ingredients_unparsed: Vec<&str> = line[0].split(",").map(|s| s.trim()).collect();
        let mut ingredients = Vec::new();
        for s in ingredients_unparsed.iter() {
            let s: Vec<&str> = s.split(" ").collect();

            ingredients.push(ChemicalUnit{
                amount: s[0].parse().unwrap(),
                id: s[1].to_string(),
            });
        }

        let product = ChemicalUnit {
            amount: product_unparsed[0].parse().unwrap(),
            id: product_unparsed[1].to_string(),
        };

        let recipe = Recipe {product, ingredients };
        reactor.add_recipe(recipe);
    }

    // the formula is ore_per_fuel * fuel <= 1 trillion ore. We are trying to maximizing fuel
    // dividing by ore_per_fuel we get fuel <= 1 trillion / ore_per_fuel
    // which is the same as fuel <= 1 trillion * fuel_per_ore

    // the inequality is there because the ore per fuel isn't same for each fuel
    // because there can be leftover products
    // so, iteratively recalculate the ore per fuel using a value for the fuel we know it is >=
    // start with 1 fuel and hope the method converges
    let mut fuel = 1;
    loop {
        reactor.do_recipe_for_product(&"FUEL".to_string(), fuel);
        let last_fuel = fuel;
        fuel = (fuel) * 1000000000000 / reactor.ore_consumed;
        if last_fuel == fuel {
            // if we follow formula and get same number, that means we have found equality
            println!("{}", last_fuel);
            break;
        } else {
            reactor.reset();
        }
    }
}
