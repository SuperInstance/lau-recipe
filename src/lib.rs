use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw materials that can be combined in recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Material {
    Stone,
    Wood,
    Crystal,
    GlowDust,
    WaterEssence,
    FireShard,
    WindLeaf,
    EarthShard,
    MoonStone,
    StarFragment,
    HeartCrystal,
    BuildKit,
    MistVial,
    SunSpark,
    IronBloom,
}

impl Material {
    /// Human-readable label for each material.
    pub fn label(&self) -> &'static str {
        match self {
            Material::Stone => "Stone",
            Material::Wood => "Wood",
            Material::Crystal => "Crystal",
            Material::GlowDust => "Glow Dust",
            Material::WaterEssence => "Water Essence",
            Material::FireShard => "Fire Shard",
            Material::WindLeaf => "Wind Leaf",
            Material::EarthShard => "Earth Shard",
            Material::MoonStone => "Moon Stone",
            Material::StarFragment => "Star Fragment",
            Material::HeartCrystal => "Heart Crystal",
            Material::BuildKit => "Build Kit",
            Material::MistVial => "Mist Vial",
            Material::SunSpark => "Sun Spark",
            Material::IronBloom => "Iron Bloom",
        }
    }
}

/// A crafting recipe — what goes in and what comes out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub inputs: Vec<(Material, usize)>,
    pub output: (Material, usize),
    pub discovered_by: Option<String>,
    pub difficulty: u32,
}

impl Recipe {
    /// Create a new recipe.
    pub fn new(
        name: &str,
        inputs: Vec<(Material, usize)>,
        output: (Material, usize),
        difficulty: u32,
    ) -> Self {
        Self {
            name: name.to_string(),
            inputs,
            output,
            discovered_by: None,
            difficulty: difficulty.clamp(1, 5),
        }
    }

    /// Mark this recipe as discovered by `discoverer`.
    pub fn discover(&mut self, discoverer: &str) {
        self.discovered_by = Some(discoverer.to_string());
    }

    /// Total number of input items needed.
    pub fn total_input_count(&self) -> usize {
        self.inputs.iter().map(|(_, count)| count).sum()
    }
}

/// A book of known and discoverable recipes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeBook {
    pub known_recipes: Vec<Recipe>,
    pub discoverable_recipes: Vec<Recipe>,
}

impl RecipeBook {
    /// Create a new empty recipe book.
    pub fn new() -> Self {
        Self {
            known_recipes: Vec::new(),
            discoverable_recipes: Vec::new(),
        }
    }

    /// Create a recipe book with the built-in recipe list.
    pub fn with_defaults() -> Self {
        Self {
            known_recipes: Vec::new(),
            discoverable_recipes: default_recipes(),
        }
    }

    /// Unlock a discoverable recipe so it can be crafted.
    pub fn discover(&mut self, recipe_name: &str, discoverer: &str) -> Option<&Recipe> {
        let pos = self
            .discoverable_recipes
            .iter()
            .position(|r| r.name == recipe_name)?;
        let mut recipe = self.discoverable_recipes.remove(pos);
        recipe.discover(discoverer);
        let name = recipe.name.clone();
        self.known_recipes.push(recipe);
        // Return a reference to the newly-inserted recipe
        self.known_recipes.iter().find(|r| r.name == name)
    }

    /// Try to craft a known recipe using materials from `inventory`.
    ///
    /// On success, returns the list of consumed inputs so the caller can remove
    /// them from their inventory. On failure, returns an error string.
    pub fn craft(
        &self,
        recipe_name: &str,
        inventory: &[(Material, usize)],
    ) -> Result<Vec<(Material, usize)>, String> {
        let recipe = self
            .known_recipes
            .iter()
            .find(|r| r.name == recipe_name)
            .ok_or_else(|| format!("Recipe '{}' is not yet known", recipe_name))?;

        if !has_sufficient_materials(recipe, inventory) {
            return Err(format!(
                "Not enough materials for recipe '{}'",
                recipe_name
            ));
        }

        Ok(recipe.inputs.clone())
    }

    /// Check whether a recipe can be crafted with the given inventory.
    pub fn can_craft(&self, recipe_name: &str, inventory: &[(Material, usize)]) -> bool {
        match self.known_recipes.iter().find(|r| r.name == recipe_name) {
            Some(recipe) => has_sufficient_materials(recipe, inventory),
            None => false,
        }
    }

    /// Search known recipes by an input material — what can I make with X?
    pub fn search_by_input(&self, material: Material) -> Vec<&Recipe> {
        self.known_recipes
            .iter()
            .filter(|r| r.inputs.iter().any(|(m, _)| *m == material))
            .collect()
    }

    /// Search all recipes (known + discoverable) by an input material.
    pub fn search_all_by_input(&self, material: Material) -> Vec<&Recipe> {
        self.known_recipes
            .iter()
            .chain(self.discoverable_recipes.iter())
            .filter(|r| r.inputs.iter().any(|(m, _)| *m == material))
            .collect()
    }

    /// Number of known recipes.
    pub fn known_count(&self) -> usize {
        self.known_recipes.len()
    }

    /// Number of undiscovered recipes.
    pub fn undiscovered_count(&self) -> usize {
        self.discoverable_recipes.len()
    }
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helper: check that inventory has enough of each required material
// ---------------------------------------------------------------------------

fn has_sufficient_materials(recipe: &Recipe, inventory: &[(Material, usize)]) -> bool {
    let mut counts: HashMap<Material, usize> = HashMap::new();
    for &(mat, count) in inventory {
        *counts.entry(mat).or_insert(0) += count;
    }
    recipe.inputs.iter().all(|(mat, need)| {
        counts.get(mat).copied().unwrap_or(0) >= *need
    })
}

// ---------------------------------------------------------------------------
// Alternative helper that works on HashMap-based inventories
// ---------------------------------------------------------------------------

fn has_sufficient_materials_map(recipe: &Recipe, inventory: &HashMap<Material, usize>) -> bool {
    recipe
        .inputs
        .iter()
        .all(|(mat, need)| inventory.get(mat).copied().unwrap_or(0) >= *need)
}

/// A player's material inventory backed by a HashMap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub materials: HashMap<Material, usize>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    /// Add `count` units of `material`.
    pub fn add(&mut self, material: Material, count: usize) {
        *self.materials.entry(material).or_insert(0) += count;
    }

    /// Remove `count` units of `material`. Returns `false` if not enough.
    pub fn remove(&mut self, material: Material, count: usize) -> bool {
        let entry = self.materials.get_mut(&material);
        match entry {
            Some(available) if *available >= count => {
                *available -= count;
                if *available == 0 {
                    self.materials.remove(&material);
                }
                true
            }
            _ => false,
        }
    }

    /// Check if at least `count` units of `material` are available.
    pub fn has(&self, material: Material, count: usize) -> bool {
        self.materials.get(&material).copied().unwrap_or(0) >= count
    }

    /// Total number of items in the inventory.
    pub fn total_items(&self) -> usize {
        self.materials.values().sum()
    }

    /// Consume the input materials of a recipe (must have been pre-checked).
    pub fn consume_for_recipe(&mut self, recipe: &Recipe) {
        for &(mat, count) in &recipe.inputs {
            self.remove(mat, count);
        }
    }

    /// Try to craft a recipe known in `book`. On success, consumes inputs and
    /// returns the output material(s). On failure, returns an error.
    pub fn craft(
        &mut self,
        recipe_name: &str,
        book: &RecipeBook,
    ) -> Result<Vec<(Material, usize)>, String> {
        let recipe = book
            .known_recipes
            .iter()
            .find(|r| r.name == recipe_name)
            .ok_or_else(|| format!("Recipe '{}' is not yet known", recipe_name))?;

        if !has_sufficient_materials_map(recipe, &self.materials) {
            return Err(format!(
                "Not enough materials for recipe '{}'",
                recipe_name
            ));
        }

        self.consume_for_recipe(recipe);
        Ok(vec![recipe.output])
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Default recipe list (16 recipes)
// ---------------------------------------------------------------------------

/// Build the standard set of discoverable recipes for the Lau platform.
pub fn default_recipes() -> Vec<Recipe> {
    vec![
        Recipe::new(
            "Glow Star",
            vec![(Material::Crystal, 1), (Material::GlowDust, 2)],
            (Material::StarFragment, 1),
            1,
        ),
        Recipe::new(
            "Tidal Stone",
            vec![(Material::WaterEssence, 2), (Material::FireShard, 1)],
            (Material::MoonStone, 1),
            2,
        ),
        Recipe::new(
            "Build Kit",
            vec![(Material::Wood, 2), (Material::Stone, 1)],
            (Material::BuildKit, 1),
            1,
        ),
        Recipe::new(
            "Heart Shard",
            vec![
                (Material::HeartCrystal, 1),
                (Material::MoonStone, 1),
                (Material::GlowDust, 1),
            ],
            (Material::StarFragment, 2),
            3,
        ),
        Recipe::new(
            "Mist Essence",
            vec![(Material::WaterEssence, 1), (Material::WindLeaf, 1)],
            (Material::MistVial, 2),
            1,
        ),
        Recipe::new(
            "Sun Spark",
            vec![(Material::FireShard, 1), (Material::GlowDust, 2)],
            (Material::SunSpark, 1),
            2,
        ),
        Recipe::new(
            "Earth Bloom",
            vec![
                (Material::EarthShard, 2),
                (Material::WaterEssence, 1),
                (Material::Wood, 1),
            ],
            (Material::IronBloom, 1),
            2,
        ),
        Recipe::new(
            "Lunar Lance",
            vec![(Material::MoonStone, 2), (Material::Crystal, 1)],
            (Material::StarFragment, 3),
            3,
        ),
        Recipe::new(
            "Breeze Vessel",
            vec![(Material::WindLeaf, 2), (Material::MistVial, 1)],
            (Material::BuildKit, 2),
            2,
        ),
        Recipe::new(
            "Forge Heart",
            vec![
                (Material::FireShard, 2),
                (Material::IronBloom, 1),
                (Material::SunSpark, 1),
            ],
            (Material::HeartCrystal, 1),
            4,
        ),
        Recipe::new(
            "Crystal Focus",
            vec![(Material::Crystal, 3), (Material::StarFragment, 1)],
            (Material::HeartCrystal, 1),
            3,
        ),
        Recipe::new(
            "Stone Wall",
            vec![(Material::Stone, 3)],
            (Material::BuildKit, 1),
            1,
        ),
        Recipe::new(
            "Glow Lantern",
            vec![(Material::GlowDust, 3), (Material::Wood, 1)],
            (Material::SunSpark, 2),
            1,
        ),
        Recipe::new(
            "Tempest Core",
            vec![
                (Material::WindLeaf, 2),
                (Material::WaterEssence, 2),
                (Material::EarthShard, 1),
            ],
            (Material::MoonStone, 3),
            4,
        ),
        Recipe::new(
            "Stellar Forge",
            vec![
                (Material::StarFragment, 2),
                (Material::FireShard, 2),
                (Material::HeartCrystal, 1),
            ],
            (Material::Crystal, 5),
            5,
        ),
        Recipe::new(
            "Primordial Clay",
            vec![
                (Material::EarthShard, 1),
                (Material::WaterEssence, 1),
            ],
            (Material::Stone, 3),
            1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Material tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_material_label() {
        assert_eq!(Material::Stone.label(), "Stone");
        assert_eq!(Material::WaterEssence.label(), "Water Essence");
        assert_eq!(Material::StarFragment.label(), "Star Fragment");
    }

    #[test]
    fn test_material_equality() {
        assert_eq!(Material::Crystal, Material::Crystal);
        assert_ne!(Material::Wood, Material::Stone);
    }

    #[test]
    fn test_material_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Material::GlowDust);
        set.insert(Material::GlowDust);
        set.insert(Material::FireShard);
        assert_eq!(set.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Recipe tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_recipe_new() {
        let r = Recipe::new(
            "Test",
            vec![(Material::Wood, 2)],
            (Material::BuildKit, 1),
            1,
        );
        assert_eq!(r.name, "Test");
        assert_eq!(r.inputs, vec![(Material::Wood, 2)]);
        assert_eq!(r.output, (Material::BuildKit, 1));
        assert!(r.discovered_by.is_none());
        assert_eq!(r.difficulty, 1);
    }

    #[test]
    fn test_recipe_difficulty_clamped() {
        let r = Recipe::new("Oops", vec![], (Material::Stone, 1), 99);
        assert_eq!(r.difficulty, 5);
        let r = Recipe::new("Safe", vec![], (Material::Stone, 1), 0);
        assert_eq!(r.difficulty, 1);
    }

    #[test]
    fn test_recipe_discover() {
        let mut r = Recipe::new("Test", vec![], (Material::Stone, 1), 1);
        r.discover("Phoenix");
        assert_eq!(r.discovered_by, Some("Phoenix".to_string()));
    }

    #[test]
    fn test_total_input_count() {
        let r = Recipe::new(
            "Multi",
            vec![(Material::Wood, 2), (Material::Stone, 3)],
            (Material::BuildKit, 1),
            1,
        );
        assert_eq!(r.total_input_count(), 5);
    }

    #[test]
    fn test_recipe_serde_roundtrip() {
        let r = Recipe::new(
            "Glow Star",
            vec![(Material::Crystal, 1), (Material::GlowDust, 2)],
            (Material::StarFragment, 1),
            1,
        );
        let json = serde_json::to_string(&r).unwrap();
        let recovered: Recipe = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.name, "Glow Star");
        assert_eq!(recovered.inputs.len(), 2);
        assert_eq!(recovered.output, (Material::StarFragment, 1));
    }

    // -----------------------------------------------------------------------
    // RecipeBook tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_recipe_book_new() {
        let book = RecipeBook::new();
        assert_eq!(book.known_count(), 0);
        assert_eq!(book.undiscovered_count(), 0);
    }

    #[test]
    fn test_recipe_book_with_defaults() {
        let book = RecipeBook::with_defaults();
        assert_eq!(book.known_count(), 0);
        assert_eq!(book.undiscovered_count(), 16);
    }

    #[test]
    fn test_discover_recipe() {
        let mut book = RecipeBook::with_defaults();
        let recipe = book.discover("Glow Star", "Alice");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().name, "Glow Star");
        assert_eq!(recipe.unwrap().discovered_by.as_deref(), Some("Alice"));
        assert_eq!(book.known_count(), 1);
        assert_eq!(book.undiscovered_count(), 15);
    }

    #[test]
    fn test_discover_nonexistent_recipe() {
        let mut book = RecipeBook::with_defaults();
        let recipe = book.discover("Fake Recipe", "Bob");
        assert!(recipe.is_none());
        assert_eq!(book.known_count(), 0);
    }

    #[test]
    fn test_craft_known_recipe_success() {
        let mut book = RecipeBook::with_defaults();
        book.discover("Glow Star", "Alice");
        let inventory = vec![(Material::Crystal, 1), (Material::GlowDust, 2)];
        let consumed = book.craft("Glow Star", &inventory);
        assert!(consumed.is_ok());
        let consumed = consumed.unwrap();
        assert_eq!(consumed.len(), 2);
    }

    #[test]
    fn test_craft_unknown_recipe_fails() {
        let book = RecipeBook::with_defaults();
        let inventory = vec![(Material::Crystal, 1)];
        let result = book.craft("Glow Star", &inventory);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("is not yet known"));
    }

    #[test]
    fn test_craft_insufficient_materials() {
        let mut book = RecipeBook::with_defaults();
        book.discover("Glow Star", "Alice");
        let inventory = vec![(Material::Crystal, 1)]; // missing GlowDust
        let result = book.craft("Glow Star", &inventory);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not enough materials"));
    }

    #[test]
    fn test_can_craft() {
        let mut book = RecipeBook::with_defaults();
        book.discover("Glow Star", "Alice");
        assert!(book.can_craft(
            "Glow Star",
            &[(Material::Crystal, 1), (Material::GlowDust, 2)]
        ));
        assert!(!book.can_craft("Glow Star", &[(Material::Crystal, 1)]));
        assert!(!book.can_craft("Unknown", &[]));
    }

    #[test]
    fn test_search_by_input() {
        let mut book = RecipeBook::with_defaults();
        book.discover("Glow Star", "Alice");
        book.discover("Build Kit", "Alice");
        // Crystal is an input for Glow Star (and Lunar Lance, but not discovered yet)
        let results = book.search_by_input(Material::Crystal);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Glow Star");
    }

    #[test]
    fn test_search_by_input_no_matches() {
        let book = RecipeBook::with_defaults();
        let results = book.search_by_input(Material::IronBloom);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_all_by_input() {
        let book = RecipeBook::with_defaults();
        // Crystal appears in "Glow Star" and "Lunar Lance" and "Crystal Focus"
        let results = book.search_all_by_input(Material::Crystal);
        let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"Glow Star"));
        assert!(names.contains(&"Lunar Lance"));
        assert!(names.contains(&"Crystal Focus"));
    }

    // -----------------------------------------------------------------------
    // Inventory tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_inventory_new() {
        let inv = Inventory::new();
        assert_eq!(inv.total_items(), 0);
    }

    #[test]
    fn test_inventory_add() {
        let mut inv = Inventory::new();
        inv.add(Material::Wood, 5);
        assert_eq!(inv.total_items(), 5);
        assert!(inv.has(Material::Wood, 5));
        assert!(inv.has(Material::Wood, 3));
        assert!(!inv.has(Material::Wood, 6));
    }

    #[test]
    fn test_inventory_add_twice() {
        let mut inv = Inventory::new();
        inv.add(Material::Crystal, 3);
        inv.add(Material::Crystal, 2);
        assert_eq!(inv.total_items(), 5);
        assert!(inv.has(Material::Crystal, 5));
    }

    #[test]
    fn test_inventory_remove_success() {
        let mut inv = Inventory::new();
        inv.add(Material::Stone, 10);
        assert!(inv.remove(Material::Stone, 3));
        assert_eq!(inv.total_items(), 7);
        assert!(inv.has(Material::Stone, 7));
    }

    #[test]
    fn test_inventory_remove_removes_empty_entry() {
        let mut inv = Inventory::new();
        inv.add(Material::Stone, 5);
        inv.remove(Material::Stone, 5);
        assert_eq!(inv.total_items(), 0);
        assert!(!inv.materials.contains_key(&Material::Stone));
    }

    #[test]
    fn test_inventory_remove_insufficient() {
        let mut inv = Inventory::new();
        inv.add(Material::Wood, 2);
        assert!(!inv.remove(Material::Wood, 5));
        assert_eq!(inv.total_items(), 2); // unchanged
    }

    #[test]
    fn test_inventory_remove_nonexistent() {
        let mut inv = Inventory::new();
        assert!(!inv.remove(Material::Crystal, 1));
    }

    #[test]
    fn test_inventory_has_nonexistent() {
        let inv = Inventory::new();
        assert!(!inv.has(Material::GlowDust, 1));
    }

    #[test]
    fn test_inventory_craft_success() {
        let mut book = RecipeBook::with_defaults();
        book.discover("Build Kit", "Alice");

        let mut inv = Inventory::new();
        inv.add(Material::Wood, 2);
        inv.add(Material::Stone, 1);

        let output = inv.craft("Build Kit", &book);
        assert!(output.is_ok());
        let output = output.unwrap();
        assert_eq!(output, vec![(Material::BuildKit, 1)]);

        // Materials consumed
        assert!(!inv.has(Material::Wood, 1));
        assert!(!inv.has(Material::Stone, 1));
        assert_eq!(inv.total_items(), 0); // BuildKit wasn't auto-added, only returned
    }

    #[test]
    fn test_inventory_craft_insufficient() {
        let mut book = RecipeBook::with_defaults();
        book.discover("Build Kit", "Alice");

        let mut inv = Inventory::new();
        inv.add(Material::Wood, 1); // only 1, need 2
        inv.add(Material::Stone, 1);

        let result = inv.craft("Build Kit", &book);
        assert!(result.is_err());
        // Inventory should be untouched
        assert!(inv.has(Material::Wood, 1));
        assert!(inv.has(Material::Stone, 1));
    }

    #[test]
    fn test_inventory_craft_unknown_recipe() {
        let book = RecipeBook::with_defaults();
        let mut inv = Inventory::new();
        inv.add(Material::Wood, 10);
        let result = inv.craft("Build Kit", &book);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not yet known"));
    }

    #[test]
    fn test_inventory_serde_roundtrip() {
        let mut inv = Inventory::new();
        inv.add(Material::StarFragment, 7);
        inv.add(Material::IronBloom, 3);
        let json = serde_json::to_string(&inv).unwrap();
        let recovered: Inventory = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.total_items(), 10);
        assert!(recovered.has(Material::StarFragment, 7));
        assert!(recovered.has(Material::IronBloom, 3));
    }

    // -----------------------------------------------------------------------
    // Full integration test: discovery → craft flow
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_crafting_flow() {
        let mut book = RecipeBook::with_defaults();
        let mut inv = Inventory::new();

        // Gather materials
        inv.add(Material::Crystal, 1);
        inv.add(Material::GlowDust, 2);

        // Discover recipe
        let recipe = book.discover("Glow Star", "Phoenix");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().discovered_by.as_deref(), Some("Phoenix"));

        // Can craft
        assert!(book.can_craft(
            "Glow Star",
            &[(Material::Crystal, 1), (Material::GlowDust, 2)]
        ));

        // Actually craft
        let output = inv.craft("Glow Star", &book);
        assert!(output.is_ok());
        let output = output.unwrap();
        assert_eq!(output, vec![(Material::StarFragment, 1)]);

        // Verify consumption
        assert!(!inv.has(Material::Crystal, 1));
        assert!(!inv.has(Material::GlowDust, 1));
        // Output is returned (not auto-added to inventory)
        assert!(!inv.has(Material::StarFragment, 1));
    }

    // -----------------------------------------------------------------------
    // Default recipe count and difficulty ranges
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_recipes_count() {
        let recipes = default_recipes();
        assert!(recipes.len() >= 15, "Expected >= 15 recipes, got {}", recipes.len());
    }

    #[test]
    fn test_all_default_recipes_valid_difficulty() {
        for r in default_recipes() {
            assert!(
                (1..=5).contains(&r.difficulty),
                "Recipe '{}' has difficulty {} (should be 1–5)",
                r.name,
                r.difficulty
            );
        }
    }

    #[test]
    fn test_no_duplicate_recipe_names() {
        let recipes = default_recipes();
        let mut names: Vec<&str> = recipes.iter().map(|r| r.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), recipes.len(), "Duplicate recipe names found");
    }
}
