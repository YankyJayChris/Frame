//! State management for the Frame runtime.
//!
//! Full Zustand-style store implementation: Task 8.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Global state container (stub — full implementation in Task 8).
pub struct State {
    data: HashMap<String, Vec<String>>,
}

impl State {
    pub fn new() -> Self {
        State { data: HashMap::new() }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: String, value: Vec<String>) {
        self.data.insert(key, value);
    }

    /// Dispatch a call string (stub — full async dispatch in Task 8).
    pub fn call(&mut self, _call: &str) {}
}

/// Internationalisation helper.
pub struct I18n {
    pub translations: HashMap<String, String>,
}

impl I18n {
    pub fn new(translations: HashMap<String, String>) -> Self {
        I18n { translations }
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.translations.get(key).map(|s| s.as_str()).unwrap_or(key)
    }
}

/// Reactive value wrapper.
pub struct Reactive<T> {
    value: Rc<RefCell<T>>,
}

impl<T: Clone> Reactive<T> {
    pub fn new(value: T) -> Self {
        Reactive { value: Rc::new(RefCell::new(value)) }
    }

    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
    }
}
