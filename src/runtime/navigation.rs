use std::collections::VecDeque;

pub struct Navigation {
    stack: VecDeque<String>,
    modal: Option<String>,
    transition: Option<String>,
}

impl Navigation {
    pub fn new() -> Self {
        let mut stack = VecDeque::new();
        stack.push_back("/profile".to_string());
        Navigation { stack, modal: None, transition: None }
    }

    pub fn push(&mut self, route: &str, transition: Option<&str>) {
        self.stack.push_back(route.to_string());
        self.transition = transition.map(String::from);
    }

    pub fn pop(&mut self) -> Option<String> {
        self.stack.pop_back()
    }

    pub fn show_modal(&mut self, route: &str, transition: Option<&str>) {
        self.modal = Some(route.to_string());
        self.transition = transition.map(String::from);
    }

    pub fn hide_modal(&mut self) {
        self.modal = None;
        self.transition = None;
    }

    pub fn current_route(&self) -> &str {
        self.modal.as_ref().unwrap_or(self.stack.back().unwrap_or(&"/profile".to_string()))
    }

    pub fn transition(&self) -> Option<&str> {
        self.transition.as_deref()
    }
}