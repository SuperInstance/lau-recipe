# lau-recipe

> Crafting recipes as mathematical compositions. Combine ingredients → transform → new creation. Recipe graphs, chain reactions, discovery mechanics.

## What This Does

The crafting system for the **Lau (Layered Agent-UI)** platform. Players combine resources from biomes into new items through recipes. But these aren't just arbitrary crafting trees — recipes are mathematical transformations. Combining "crystal" + "heat" follows actual phase transition logic. Mixing reagents produces outputs based on conservation laws.

Every recipe is a function: inputs → output. Recipe chains are function composition. Discovery = finding new compositions that the player hasn't tried yet.

## Quick Start

```rust
use lau_recipe::{RecipeBook, Recipe, Ingredient};

let mut book = RecipeBook::new();

// Define a recipe: 2 wood + 1 stone → workbench
book.register(Recipe::new("workbench")
    .input(Ingredient::new("wood", 2))
    .input(Ingredient::new("stone", 1))
    .output(Ingredient::new("workbench", 1))
);

// More complex recipe
book.register(Recipe::new("crystal-sword")
    .input(Ingredient::new("crystal", 3))
    .input(Ingredient::new("obsidian", 1))
    .input(Ingredient::new("handle", 1))
    .output(Ingredient::new("crystal-sword", 1))
    .biome_required("CrystalCaves")  // only craftable here
);

// Check what can be crafted from inventory
let inventory = vec![
    Ingredient::new("wood", 5),
    Ingredient::new("stone", 3),
];
let craftable = book.what_can_craft(&inventory);
// Returns: [("workbench", craftable: true)]

// Craft it
let result = book.craft("workbench", &mut inventory)?;
// inventory now: wood=3, stone=2, workbench=1
```

## API Reference

### Recipe

| Method | Description |
|--------|-------------|
| `Recipe::new(name)` | Create recipe |
| `.input(ingredient)` | Required input |
| `.output(ingredient)` | What it produces |
| `.biome_required(biome)` | Location restriction |
| `.skill_required(skill, level)` | Skill gate |

### RecipeBook

| Method | Description |
|--------|-------------|
| `RecipeBook::new()` | Empty book |
| `book.register(recipe)` | Add recipe |
| `book.craft(name, inventory)` | Execute recipe |
| `book.what_can_craft(inventory)` | Check all possibilities |
| `book.recipes_using(item)` | Find recipes by ingredient |
| `book.chain_graph()` | Build full crafting DAG |

### Ingredient

| Method | Description |
|--------|-------------|
| `Ingredient::new(name, count)` | Create |
| `ing.name()` / `ing.count()` | Accessors |

## Testing

35 tests: recipe registration, crafting, inventory management, biome restrictions, skill gates, insufficient materials, chain graphs, discovery mechanics.

## Part of the Lau Platform

Part of the Lau game engine: [lau-git-world](https://github.com/SuperInstance/lau-git-world) · [lau-quest](https://github.com/SuperInstance/lau-quest) · [lau-biome](https://github.com/SuperInstance/lau-biome) · [lau-spatial](https://github.com/SuperInstance/lau-spatial) · [lau-audio](https://github.com/SuperInstance/lau-audio) · [lau-scheduler](https://github.com/SuperInstance/lau-scheduler) · [lau-memory-arena](https://github.com/SuperInstance/lau-memory-arena) · [lau-genealogy](https://github.com/SuperInstance/lau-genealogy) · **lau-recipe**

## License

MIT
