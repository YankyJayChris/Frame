use std::collections::{VecDeque, HashMap};
use crate::parser::Page;

pub struct Navigation {
    stack: VecDeque<String>,
    modal: Option<String>,
    transition: Option<String>,
    params: HashMap<String, HashMap<String, String>>,
    pages: Vec<Page>,
}

impl Navigation {
    pub fn new() -> Self {
        let mut stack = VecDeque::new();
        stack.push_back("/home".to_string());
        Navigation {
            stack,
            modal: None,
            transition: None,
            params: HashMap::new(),
            pages: Vec::new(),
        }
    }

    pub fn set_pages(&mut self, pages: Vec<Page>) {
        self.pages = pages;
    }

    pub fn push(&mut self, route: &str, transition: Option<&str>) {
        let (base_route, params) = parse_route(route);
        self.stack.push_back(base_route.to_string());
        if !params.is_empty() {
            self.params.insert(base_route.to_string(), params);
        }
        self.transition = transition.map(String::from);
    }

    pub fn pop(&mut self) -> Option<String> {
        if let Some(route) = self.stack.pop_back() {
            self.params.remove(&route);
            Some(route)
        } else {
            None
        }
    }

    pub fn show_modal(&mut self, route: &str, transition: Option<&str>) {
        let (base_route, params) = parse_route(route);
        self.modal = Some(base_route.to_string());
        if !params.is_empty() {
            self.params.insert(base_route.to_string(), params);
        }
        self.transition = transition.map(String::from);
    }

    pub fn hide_modal(&mut self) {
        if let Some(route) = &self.modal {
            self.params.remove(route);
        }
        self.modal = None;
        self.transition = None;
    }

    pub fn current_route(&self) -> &str {
        self.modal.as_ref().unwrap_or(self.stack.back().unwrap_or(&"/home".to_string()))
    }

    pub fn current_page(&self) -> Option<&Page> {
        self.pages.iter().find(|p| p.route == self.current_route())
    }

    pub fn transition(&self) -> Option<&str> {
        self.transition.as_deref()
    }

    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(self.current_route()).and_then(|p| p.get(key))
    }
}

fn parse_route(route: &str) -> (&str, HashMap<String, String>) {
    let mut params = HashMap::new();
    if route.contains('?') {
        let parts: Vec<&str> = route.split('?').collect();
        let base_route = parts[0];
        if parts.len() > 1 {
            for param in parts[1].split('&') {
                let kv: Vec<&str> = param.split('=').collect();
                if kv.len() == 2 {
                    params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }
        (base_route, params)
    } else {
        (route, params)
    }
}