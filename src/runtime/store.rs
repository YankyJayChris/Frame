//! Zustand-style reactive store system for the Frame runtime.

use std::collections::{HashMap, HashSet};

// ─── Value type ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum StoreValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<StoreValue>),
    Object(HashMap<String, StoreValue>),
}

impl Default for StoreValue {
    fn default() -> Self { StoreValue::Null }
}

// ─── PersistStrategy ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PersistStrategy {
    Local,
    Secure,
}

const SENSITIVE_PATTERNS: &[&str] = &[
    "token", "secret", "password", "key", "credential", "auth",
];

pub fn auto_persist_strategy(field_name: &str, declared: PersistStrategy) -> PersistStrategy {
    let lower = field_name.to_lowercase();
    if SENSITIVE_PATTERNS.iter().any(|p| lower.contains(p)) {
        PersistStrategy::Secure
    } else {
        declared
    }
}

// ─── ComponentRef ─────────────────────────────────────────────────────────────

pub type ComponentRef = String;

// ─── FieldSubscribers ─────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct FieldSubscribers {
    pub subscriptions: HashMap<String, HashSet<ComponentRef>>,
}

impl FieldSubscribers {
    pub fn new() -> Self { Self::default() }

    pub fn subscribe(&mut self, field: &str, component: ComponentRef) {
        self.subscriptions.entry(field.to_string()).or_default().insert(component);
    }

    pub fn unsubscribe_all(&mut self, component: &str) {
        for subscribers in self.subscriptions.values_mut() {
            subscribers.remove(component);
        }
    }

    pub fn get_subscribers(&self, field: &str) -> Vec<ComponentRef> {
        self.subscriptions.get(field).map(|s| s.iter().cloned().collect()).unwrap_or_default()
    }

    pub fn total_subscribers(&self) -> usize {
        self.subscriptions.values().map(|s| s.len()).sum()
    }
}

// ─── RenderScheduler ─────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct RenderScheduler {
    pub scheduled: Vec<ComponentRef>,
}

impl RenderScheduler {
    pub fn new() -> Self { Self::default() }

    pub fn schedule(&mut self, component: ComponentRef) {
        self.scheduled.push(component);
    }

    pub fn drain(&mut self) -> Vec<ComponentRef> {
        std::mem::take(&mut self.scheduled)
    }

    pub fn pending_count(&self) -> usize {
        self.scheduled.len()
    }
}

// ─── PersistenceLayer ────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct PersistenceLayer {
    pub local: HashMap<String, String>,
    pub secure: HashMap<String, String>,
}

impl PersistenceLayer {
    pub fn new() -> Self { Self::default() }

    pub fn write(&mut self, field: &str, value: &StoreValue, strategy: &PersistStrategy) {
        let serialised = serialise_value(value);
        match strategy {
            PersistStrategy::Local  => { self.local.insert(field.to_string(), serialised); }
            PersistStrategy::Secure => { self.secure.insert(field.to_string(), serialised); }
        }
    }

    pub fn read(&self, field: &str, strategy: &PersistStrategy) -> Option<StoreValue> {
        let serialised = match strategy {
            PersistStrategy::Local  => self.local.get(field)?,
            PersistStrategy::Secure => self.secure.get(field)?,
        };
        Some(deserialise_value(serialised))
    }
}

fn serialise_value(v: &StoreValue) -> String {
    match v {
        StoreValue::Null      => "null".to_string(),
        StoreValue::Bool(b)   => b.to_string(),
        StoreValue::Int(n)    => n.to_string(),
        StoreValue::Float(f)  => f.to_string(),
        StoreValue::Str(s)    => s.clone(),
        StoreValue::List(_)   => "[]".to_string(),
        StoreValue::Object(_) => "{}".to_string(),
    }
}

fn deserialise_value(s: &str) -> StoreValue {
    if s == "null"  { return StoreValue::Null; }
    if s == "true"  { return StoreValue::Bool(true); }
    if s == "false" { return StoreValue::Bool(false); }
    if let Ok(n) = s.parse::<i64>()  { return StoreValue::Int(n); }
    if let Ok(f) = s.parse::<f64>()  { return StoreValue::Float(f); }
    StoreValue::Str(s.to_string())
}

// ─── DevtoolsEvent ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DevtoolsEvent {
    pub store_name: String,
    pub field: String,
    pub old_value: StoreValue,
    pub new_value: StoreValue,
    pub timestamp_ms: u64,
}

// ─── StoreSlice ───────────────────────────────────────────────────────────────

pub struct StoreSlice {
    pub name: String,
    pub fields: HashMap<String, StoreValue>,
    pub persist: HashMap<String, PersistStrategy>,
    pub subscribers: FieldSubscribers,
    pub scheduler: RenderScheduler,
    pub storage: PersistenceLayer,
    pub devtools_log: Vec<DevtoolsEvent>,
    pub tick: u64,
}

impl StoreSlice {
    pub fn new(
        name: impl Into<String>,
        fields: HashMap<String, StoreValue>,
        persist_config: HashMap<String, PersistStrategy>,
    ) -> Self {
        let persist = persist_config.into_iter().map(|(k, v)| {
            let effective = auto_persist_strategy(&k, v);
            (k, effective)
        }).collect();

        StoreSlice {
            name: name.into(),
            fields,
            persist,
            subscribers: FieldSubscribers::new(),
            scheduler: RenderScheduler::new(),
            storage: PersistenceLayer::new(),
            devtools_log: Vec::new(),
            tick: 0,
        }
    }

    pub fn mutate(&mut self, field: &str, value: StoreValue) {
        let old_value = self.fields.get(field).cloned().unwrap_or(StoreValue::Null);
        self.fields.insert(field.to_string(), value.clone());

        self.tick += 1;
        self.devtools_log.push(DevtoolsEvent {
            store_name: self.name.clone(),
            field: field.to_string(),
            old_value,
            new_value: value.clone(),
            timestamp_ms: self.tick,
        });

        if let Some(strategy) = self.persist.get(field) {
            let strategy = strategy.clone();
            self.storage.write(field, &value, &strategy);
        }

        let subscribers = self.subscribers.get_subscribers(field);
        for component in subscribers {
            self.scheduler.schedule(component);
        }
    }

    pub fn subscribe(&mut self, field: &str, component: ComponentRef) {
        self.subscribers.subscribe(field, component);
    }

    pub fn restore(&mut self) {
        let persisted_fields: Vec<(String, PersistStrategy)> =
            self.persist.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        for (field, strategy) in persisted_fields {
            if let Some(value) = self.storage.read(&field, &strategy) {
                self.fields.insert(field, value);
            }
        }
    }

    pub fn devtools_snapshot(&self) -> HashMap<String, StoreValue> {
        self.fields.clone()
    }

    pub fn devtools_events(&self) -> &[DevtoolsEvent] {
        &self.devtools_log
    }

    pub fn drain_renders(&mut self) -> Vec<ComponentRef> {
        self.scheduler.drain()
    }

    pub fn pending_renders(&self) -> usize {
        self.scheduler.pending_count()
    }
}

// ─── StoreRegistry ───────────────────────────────────────────────────────────

pub struct StoreRegistry {
    pub slices: HashMap<String, StoreSlice>,
}

impl StoreRegistry {
    pub fn new() -> Self {
        StoreRegistry { slices: HashMap::new() }
    }

    pub fn register(&mut self, slice: StoreSlice) {
        self.slices.insert(slice.name.clone(), slice);
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut StoreSlice> {
        self.slices.get_mut(name)
    }

    pub fn get(&self, name: &str) -> Option<&StoreSlice> {
        self.slices.get(name)
    }

    pub fn restore_all(&mut self) {
        for slice in self.slices.values_mut() {
            slice.restore();
        }
    }

    pub fn devtools_all(&self) -> HashMap<String, HashMap<String, StoreValue>> {
        self.slices.iter().map(|(name, slice)| {
            (name.clone(), slice.devtools_snapshot())
        }).collect()
    }

    pub fn dispatch_action(&mut self, store_name: &str, _action_name: &str, _args: Vec<StoreValue>) {
        if let Some(slice) = self.slices.get(store_name) {
            let _ = slice.devtools_snapshot();
        }
    }
}

impl Default for StoreRegistry {
    fn default() -> Self { Self::new() }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_auth_store() -> StoreSlice {
        let mut fields = HashMap::new();
        fields.insert("token".to_string(), StoreValue::Str("".to_string()));
        fields.insert("is_loading".to_string(), StoreValue::Bool(false));
        fields.insert("user".to_string(), StoreValue::Null);

        let mut persist = HashMap::new();
        persist.insert("token".to_string(), PersistStrategy::Local);

        StoreSlice::new("AuthStore", fields, persist)
    }

    #[test]
    fn test_n_subscribers_trigger_n_rerenders() {
        let mut slice = make_auth_store();
        for i in 0..5 {
            slice.subscribe("token", format!("Component{}", i));
        }
        slice.mutate("token", StoreValue::Str("abc".to_string()));
        let renders = slice.drain_renders();
        assert_eq!(renders.len(), 5, "Expected exactly 5 re-renders, got {}", renders.len());
    }

    #[test]
    fn test_zero_subscribers_triggers_zero_rerenders() {
        let mut slice = make_auth_store();
        slice.mutate("token", StoreValue::Str("abc".to_string()));
        let renders = slice.drain_renders();
        assert_eq!(renders.len(), 0);
    }

    #[test]
    fn test_mutation_only_notifies_field_subscribers() {
        let mut slice = make_auth_store();
        slice.subscribe("token", "Component1".to_string());
        slice.subscribe("is_loading", "Component2".to_string());

        slice.mutate("token", StoreValue::Str("xyz".to_string()));
        let renders = slice.drain_renders();
        assert_eq!(renders.len(), 1);
        assert!(renders.contains(&"Component1".to_string()));
        assert!(!renders.contains(&"Component2".to_string()));
    }

    #[test]
    fn test_sensitive_token_field_auto_upgrades_to_secure() {
        let slice = make_auth_store();
        assert_eq!(slice.persist.get("token"), Some(&PersistStrategy::Secure));
    }

    #[test]
    fn test_nonsensitive_field_stays_local() {
        let mut fields = HashMap::new();
        fields.insert("theme".to_string(), StoreValue::Str("dark".to_string()));
        let mut persist = HashMap::new();
        persist.insert("theme".to_string(), PersistStrategy::Local);
        let slice = StoreSlice::new("SettingsStore", fields, persist);
        assert_eq!(slice.persist.get("theme"), Some(&PersistStrategy::Local));
    }

    #[test]
    fn test_password_field_auto_upgrades_to_secure() {
        let persist = HashMap::from([("password".to_string(), PersistStrategy::Local)]);
        let slice = StoreSlice::new("UserStore", HashMap::new(), persist);
        assert_eq!(slice.persist.get("password"), Some(&PersistStrategy::Secure));
    }

    #[test]
    fn test_credential_field_auto_upgrades_to_secure() {
        let persist = HashMap::from([("api_credential".to_string(), PersistStrategy::Local)]);
        let slice = StoreSlice::new("ApiStore", HashMap::new(), persist);
        assert_eq!(slice.persist.get("api_credential"), Some(&PersistStrategy::Secure));
    }

    #[test]
    fn test_auto_persist_strategy_token() {
        assert_eq!(auto_persist_strategy("token", PersistStrategy::Local), PersistStrategy::Secure);
    }

    #[test]
    fn test_auto_persist_strategy_auth_token() {
        assert_eq!(auto_persist_strategy("auth_token", PersistStrategy::Local), PersistStrategy::Secure);
    }

    #[test]
    fn test_auto_persist_strategy_username_stays_local() {
        assert_eq!(auto_persist_strategy("username", PersistStrategy::Local), PersistStrategy::Local);
    }

    #[test]
    fn test_persist_and_restore() {
        let mut slice = make_auth_store();
        slice.mutate("token", StoreValue::Str("abc123".to_string()));

        let storage = slice.storage.clone();
        let persist = slice.persist.clone();
        let mut new_slice = StoreSlice {
            name: "AuthStore".to_string(),
            fields: {
                let mut f = HashMap::new();
                f.insert("token".to_string(), StoreValue::Str("".to_string()));
                f
            },
            persist,
            subscribers: FieldSubscribers::new(),
            scheduler: RenderScheduler::new(),
            storage,
            devtools_log: Vec::new(),
            tick: 0,
        };

        new_slice.restore();
        assert_eq!(new_slice.fields.get("token"), Some(&StoreValue::Str("abc123".to_string())));
    }

    #[test]
    fn test_mutate_from_different_contexts_produces_same_state() {
        let mut slice_a = make_auth_store();
        let mut slice_b = make_auth_store();
        slice_a.mutate("token", StoreValue::Str("xyz".to_string()));
        slice_b.mutate("token", StoreValue::Str("xyz".to_string()));
        assert_eq!(slice_a.fields.get("token"), slice_b.fields.get("token"));
    }

    #[test]
    fn test_devtools_snapshot_reflects_current_state() {
        let mut slice = make_auth_store();
        slice.mutate("token", StoreValue::Str("test_token".to_string()));
        let snapshot = slice.devtools_snapshot();
        assert_eq!(snapshot.get("token"), Some(&StoreValue::Str("test_token".to_string())));
    }

    #[test]
    fn test_devtools_log_records_mutations() {
        let mut slice = make_auth_store();
        slice.mutate("token", StoreValue::Str("t1".to_string()));
        slice.mutate("is_loading", StoreValue::Bool(true));
        let events = slice.devtools_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].field, "token");
        assert_eq!(events[1].field, "is_loading");
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = StoreRegistry::new();
        registry.register(make_auth_store());
        assert!(registry.get("AuthStore").is_some());
    }

    #[test]
    fn test_registry_restore_all() {
        let mut registry = StoreRegistry::new();
        registry.register(make_auth_store());
        registry.restore_all();
    }

    #[test]
    fn test_registry_devtools_all() {
        let mut registry = StoreRegistry::new();
        registry.register(make_auth_store());
        let all = registry.devtools_all();
        assert!(all.contains_key("AuthStore"));
    }

    #[test]
    fn test_field_subscribers_subscribe_and_notify() {
        let mut subs = FieldSubscribers::new();
        subs.subscribe("token", "A".to_string());
        subs.subscribe("token", "B".to_string());
        subs.subscribe("name", "C".to_string());
        let token_subs = subs.get_subscribers("token");
        assert_eq!(token_subs.len(), 2);
        assert!(token_subs.contains(&"A".to_string()));
        assert!(token_subs.contains(&"B".to_string()));
        let name_subs = subs.get_subscribers("name");
        assert_eq!(name_subs.len(), 1);
    }

    #[test]
    fn test_field_subscribers_unsubscribe_all() {
        let mut subs = FieldSubscribers::new();
        subs.subscribe("token", "A".to_string());
        subs.subscribe("name", "A".to_string());
        subs.unsubscribe_all("A");
        assert_eq!(subs.get_subscribers("token").len(), 0);
        assert_eq!(subs.get_subscribers("name").len(), 0);
    }
}
