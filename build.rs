use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct BaseTheme {
    #[serde(rename = "tokenColors")]
    token_colors: Vec<TokenColor>,
}

#[derive(Deserialize)]
struct TokenColor {
    name: Option<String>,
}

fn main() {
    println!("cargo:rerun-if-changed=data/modern_dark_token_color.json");

    let json_str = fs::read_to_string("data/modern_dark_token_color.json")
        .expect("Failed to read JSON in build script");

    let theme: BaseTheme =
        serde_json::from_str(&json_str).expect("Failed to parse JSON in build script");

    let mut enum_variants = String::new();
    let mut match_arms = String::new();
    let mut as_str_arms = String::new();

    let mut seen_variants = HashSet::new();
    let mut seen_names = HashSet::new();

    for token in theme.token_colors {
        if let Some(name) = token.name {
            if !seen_names.insert(name.clone()) {
                unreachable!();
            }

            let variant_name = name
                .replace(&[':', '/', '-', '(', ')', '&'][..], " ")
                .split_whitespace()
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<String>();

            if seen_variants.insert(variant_name.clone()) {
                enum_variants.push_str(&format!("    {},\n", variant_name));
                as_str_arms.push_str(&format!(
                    "            Self::{} => {:?},\n",
                    variant_name, name
                ));
            }
            match_arms.push_str(&format!(
                "            {:?} => Ok(TokenName::{}),\n",
                name, variant_name
            ));
        }
    }

    let generated_code = format!(
        r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenName {{
{enum_variants}
}}

impl std::str::FromStr for TokenName {{
    type Err = std::convert::Infallible;

    fn from_str(name: &str) -> std::result::Result<TokenName, Self::Err> {{
        match name {{
{match_arms}
            _ => unreachable!(),
        }}
    }}
}}

impl TokenName {{
    pub const fn as_str(&self) -> &'static str {{
        match self {{
{as_str_arms}
        }}
    }}
}}
"#,
        enum_variants = enum_variants,
        match_arms = match_arms
    );

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("base_token_colors.rs");
    fs::write(&dest_path, generated_code).unwrap();
}
