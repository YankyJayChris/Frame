use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref ICONS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("settings", "assets/icons/settings.svg");
        m.insert("search", "assets/icons/search.svg");
        m.insert("list", "assets/icons/list.svg");
        m.insert("add", "assets/icons/add.svg");
        m.insert("home", "assets/icons/home.svg");
        m.insert("services", "assets/icons/services.svg");
        m.insert("profile", "assets/icons/profile.svg");
        m
    };
}

pub fn get_icon(name: &str) -> &'static str {
    ICONS.get(name).unwrap_or(&"assets/icons/default.svg")
}