use rust_embed::RustEmbed;
use serde_json::Value;

#[derive(RustEmbed)]
#[folder = "locales/"]
struct Asset;

pub struct I18n {
    texts: Value,
    pub current_lang: String,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        let path = format!("{}.json", lang);
        let content = if let Some(file) = Asset::get(&path) {
            std::str::from_utf8(file.data.as_ref())
                .expect("UTF-8")
                .to_string()
        } else {
            r#"{"processing": "...", "wait": "..."}"#.to_string()
        };
        I18n {
            texts: serde_json::from_str(&content).expect("JSON"),
            current_lang: lang.to_string(),
        }
    }

    pub fn get(&self, key: &str) -> String {
        self.texts
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(key)
            .to_string()
    }

    pub fn get_args(&self, key: &str, args: &[String]) -> String {
        let mut s = self.get(key);
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            s = s.replace(&placeholder, arg);
        }
        s
    }
}
