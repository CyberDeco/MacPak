//! Recipe system for workbench project scaffolding
//!
//! Recipes are bundled TOML files describing what a mod type needs:
//! directory structure, generated files, and manual/optional files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A recipe describing a mod type and its required structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub recipe: RecipeMeta,
    #[serde(default)]
    pub variables: Vec<RecipeVariable>,
    pub structure: RecipeStructure,
    #[serde(default)]
    pub files: Vec<RecipeFile>,
}

/// Recipe metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

/// A variable that a recipe can request from the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeVariable {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub var_type: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: String,
}

/// Directory structure to scaffold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStructure {
    pub directories: Vec<String>,
}

/// A file entry in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFile {
    pub path: String,
    pub kind: FileKind,
    #[serde(default)]
    pub generator: Option<String>,
    pub description: String,
    #[serde(default)]
    pub hint: Option<String>,
}

/// Whether a file is auto-generated, requires manual creation, or is optional
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Generated,
    Manual,
    Optional,
}

// Bundled recipe TOML files
const GENERIC_RECIPE: &str = include_str!("recipes/generic.toml");
const EQUIPMENT_RECIPE: &str = include_str!("recipes/equipment.toml");
const SPELL_RECIPE: &str = include_str!("recipes/spell.toml");
const DYES_RECIPE: &str = include_str!("recipes/dyes.toml");
const HAIR_RECIPE: &str = include_str!("recipes/hair.toml");
const CLASS_RECIPE: &str = include_str!("recipes/class.toml");

/// Load all bundled recipes
pub fn load_bundled_recipes() -> Vec<Recipe> {
    let sources = [
        GENERIC_RECIPE,
        EQUIPMENT_RECIPE,
        SPELL_RECIPE,
        DYES_RECIPE,
        HAIR_RECIPE,
        CLASS_RECIPE,
    ];
    sources
        .iter()
        .filter_map(|src| match toml::from_str::<Recipe>(src) {
            Ok(recipe) => Some(recipe),
            Err(e) => {
                tracing::warn!("Failed to parse bundled recipe: {}", e);
                None
            }
        })
        .collect()
}

/// Find a bundled recipe by its id
pub fn find_recipe(id: &str) -> Option<Recipe> {
    load_bundled_recipes()
        .into_iter()
        .find(|r| r.recipe.id == id)
}

/// Substitute template variables in a string.
///
/// Replaces `{{key}}` with the corresponding value from the vars map.
pub fn substitute(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_bundled_recipes() {
        let recipes = load_bundled_recipes();
        assert_eq!(recipes.len(), 6);
        assert!(recipes.iter().any(|r| r.recipe.id == "generic"));
        assert!(recipes.iter().any(|r| r.recipe.id == "equipment"));
        assert!(recipes.iter().any(|r| r.recipe.id == "spell"));
        assert!(recipes.iter().any(|r| r.recipe.id == "dyes"));
        assert!(recipes.iter().any(|r| r.recipe.id == "hair"));
        assert!(recipes.iter().any(|r| r.recipe.id == "class"));
    }

    #[test]
    fn test_substitute() {
        let mut vars = HashMap::new();
        vars.insert("mod_name".to_string(), "MyCoolMod".to_string());
        vars.insert("uuid".to_string(), "abc-123".to_string());

        assert_eq!(
            substitute("Mods/{{mod_name}}/meta.lsx", &vars),
            "Mods/MyCoolMod/meta.lsx"
        );
        assert_eq!(substitute("no-vars-here", &vars), "no-vars-here");
        assert_eq!(
            substitute("{{mod_name}}/{{uuid}}", &vars),
            "MyCoolMod/abc-123"
        );
    }

    #[test]
    fn test_find_recipe() {
        assert!(find_recipe("generic").is_some());
        assert!(find_recipe("nonexistent").is_none());
    }
}
