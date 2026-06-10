use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct RelationRecipe {
    pub source: String,
    pub target: String,
    pub result: String,
    #[serde(default)]
    pub near: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImpactRecipe {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub result: String,
    #[serde(default)]
    pub extra_result: String,
    #[serde(default)]
    pub extra_offset: (i8, i8),
    #[serde(default)]
    pub handler_id: String,
    #[serde(default = "default_hits")]
    pub hits_required: u32,
    #[serde(default)]
    pub consumes_target: bool,
    #[serde(default)]
    pub consumes_source: bool,
}

fn default_hits() -> u32 {
    2
}

#[derive(Debug, Clone, Default)]
pub struct RecipeBook {
    pub relations: Vec<RelationRecipe>,
    pub impacts: Vec<ImpactRecipe>,
}

impl RecipeBook {
    pub fn load_embedded() -> Self {
        Self::from_ron(include_str!("../../assets/relations.ron"), include_str!("../../assets/impacts.ron"))
            .unwrap_or_else(|_| Self::fallback())
    }

    pub fn from_ron(relations: &str, impacts: &str) -> Result<Self, ron::Error> {
        Ok(Self {
            relations: ron::from_str(relations)?,
            impacts: ron::from_str(impacts)?,
        })
    }

    pub fn from_files(root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let rel = std::fs::read_to_string(root.join("relations.ron"))?;
        let imp = std::fs::read_to_string(root.join("impacts.ron"))?;
        Ok(Self::from_ron(&rel, &imp)?)
    }

    pub fn find_relation(&self, a: &str, b: &str) -> Option<&RelationRecipe> {
        self.relations
            .iter()
            .find(|r| (r.source == a && r.target == b) || (r.source == b && r.target == a))
    }

    pub fn resolve_impact(&self, source: &str, target: &str) -> Option<&ImpactRecipe> {
        if matches!(source, "knife" | "axe" | "spear" | "shard") && target.ends_with("Corpse") {
            return self.impacts.iter().find(|r| r.handler_id == "butcher_corpse");
        }
        if matches!(
            (source, target),
            ("knife" | "axe" | "spear" | "shard", "sheep" | "deer")
        ) {
            return self.impacts.iter().find(|r| r.handler_id == "sheep_butcher");
        }
        for recipe in &self.impacts {
            if recipe_matches(recipe.source.as_str(), source)
                && recipe_matches(recipe.target.as_str(), target)
            {
                return Some(recipe);
            }
        }
        None
    }

    fn fallback() -> Self {
        Self {
            relations: vec![
                RelationRecipe {
                    source: "twig".into(),
                    target: "shard".into(),
                    result: "spear".into(),
                    near: false,
                },
                RelationRecipe {
                    source: "tri".into(),
                    target: "wood".into(),
                    result: "axe".into(),
                    near: false,
                },
            ],
            impacts: vec![
                ImpactRecipe {
                    source: "stone".into(),
                    target: "stone".into(),
                    result: "shard".into(),
                    extra_result: String::new(),
                    extra_offset: (1, 0),
                    handler_id: String::new(),
                    hits_required: 2,
                    consumes_target: false,
                    consumes_source: false,
                },
                ImpactRecipe {
                    source: "wood_like".into(),
                    target: "shard".into(),
                    result: "fire".into(),
                    extra_result: String::new(),
                    extra_offset: (0, 0),
                    handler_id: String::new(),
                    hits_required: 2,
                    consumes_target: false,
                    consumes_source: false,
                },
            ],
        }
    }
}

fn recipe_matches(pattern: &str, type_name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.ends_with("_like") {
        let base = &pattern[..pattern.len() - 5];
        return type_name.contains(base) || matches!(base, "wood" if type_name == "wood" || type_name == "twig");
    }
    pattern == type_name
}
