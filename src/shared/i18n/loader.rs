use std::collections::HashMap;

const DEFAULT_EN: &str = include_str!("../../../lang/en.json");
const DEFAULT_DE: &str = include_str!("../../../lang/de.json");

#[derive(Clone, Debug)]
pub struct I18n {
    strings: HashMap<String, String>,
}

impl I18n {
    pub fn load() -> Self {
        let code = detect_language_code();

        let content = std::fs::read_to_string(format!("lang/{code}.json"))
            .ok()
            .or_else(|| default_for_code(&code).map(ToOwned::to_owned))
            .unwrap_or_else(|| DEFAULT_EN.to_owned());

        match serde_json::from_str::<HashMap<String, String>>(&content) {
            Ok(strings) => Self { strings },
            Err(_) => Self::from_default_en(),
        }
    }

    pub fn text(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("{{{key}}}"))
    }

    pub fn text_with(&self, key: &str, placeholders: &[(&str, String)]) -> String {
        let mut template = self.text(key);

        for (placeholder, value) in placeholders {
            let token = format!("{{{{{placeholder}}}}}");
            template = template.replace(&token, value);
        }

        template
    }

    fn from_default_en() -> Self {
        let strings = serde_json::from_str::<HashMap<String, String>>(DEFAULT_EN)
            .unwrap_or_else(|_| HashMap::new());
        Self { strings }
    }
}

fn detect_language_code() -> String {
    std::env::var("NALUMINALINK_LANG")
        .ok()
        .or_else(|| std::env::var("LANG").ok())
        .map(|raw| {
            raw.split(&['_', '-', '.'][..])
                .next()
                .unwrap_or("en")
                .to_lowercase()
        })
        .filter(|code| !code.is_empty())
        .unwrap_or_else(|| String::from("en"))
}

fn default_for_code(code: &str) -> Option<&'static str> {
    match code {
        "de" => Some(DEFAULT_DE),
        "en" => Some(DEFAULT_EN),
        _ => None,
    }
}
