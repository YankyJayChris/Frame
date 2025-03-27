use std::collections::HashMap;
use tokio::sync::mpsc;
use rusqlite::{Connection, params};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};
use std::rc::Rc;
use std::cell::RefCell;

pub struct State {
    data: HashMap<String, Vec<String>>,
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
    db: Connection,
    subscribers: HashMap<String, Vec<Box<dyn Fn()>>>,
}

impl State {
    pub fn new() -> Self {
        let db = Connection::open("frame_state.db").unwrap();
        db.execute(
            "CREATE TABLE IF NOT EXISTS state (key TEXT PRIMARY KEY, value TEXT, timestamp INTEGER)",
            [],
        ).unwrap();
        let mut data = HashMap::new();
        let mut stmt = db.prepare("SELECT key, value FROM state").unwrap();
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        for row in rows {
            let (key, value): (String, String) = row.unwrap();
            data.insert(key, serde_json::from_str(&value).unwrap());
        }
        let (sender, receiver) = mpsc::channel(100);
        State {
            data,
            sender,
            receiver,
            db,
            subscribers: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: String, value: Vec<String>) {
        let value_str = serde_json::to_string(&value).unwrap();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.db.execute(
            "INSERT OR REPLACE INTO state (key, value, timestamp) VALUES (?1, ?2, ?3)",
            params![&key, &value_str, timestamp],
        ).unwrap();
        self.data.insert(key.clone(), value);
        if let Some(subs) = self.subscribers.get(&key) {
            for sub in subs {
                sub();
            }
        }
    }

    pub fn call(&mut self, call: &str) {
        let sender = self.sender.clone();
        let call_str = call.to_string();
        tokio::spawn(async move {
            if call_str.contains("fetch") {
                let parts: Vec<&str> = call_str.split("fetch:(").collect();
                let args: Vec<&str> = parts[1].trim_end_matches(')').split(',').collect();
                let url = args[0].trim_matches('"');
                let method = args[1].split("method:").nth(1).unwrap().trim_matches('"');
                let client = reqwest::Client::new();
                let resp = match method {
                    "GET" => client.get(url).send().await.unwrap().json::<Vec<HashMap<String, String>>>().await.unwrap(),
                    "POST" => client.post(url).send().await.unwrap().json::<Vec<HashMap<String, String>>>().await.unwrap(),
                    _ => panic!("Unsupported method"),
                };
                let posts = resp.iter().map(|p| p["title"].clone()).collect::<Vec<_>>();
                sender.send(format!("set:posts:{}", serde_json::to_string(&posts).unwrap())).await.unwrap();
            }
        });

        if let Ok(Some(msg)) = self.receiver.try_recv() {
            if msg.starts_with("set:") {
                let parts: Vec<&str> = msg.split(':').collect();
                self.set(parts[1].to_string(), serde_json::from_str(parts[2]).unwrap());
            }
        }
    }

    pub fn subscribe(&mut self, key: &str, callback: Box<dyn Fn()>) {
        self.subscribers.entry(key.to_string()).or_insert_with(Vec::new).push(callback);
    }
}

pub struct Reactive<T> {
    value: Rc<RefCell<T>>,
    subscribers: Vec<Box<dyn Fn()>>,
}

impl<T: Clone> Reactive<T> {
    pub fn new(value: T) -> Self {
        Reactive {
            value: Rc::new(RefCell::new(value)),
            subscribers: Vec::new(),
        }
    }

    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        for sub in &self.subscribers {
            sub();
        }
    }

    pub fn subscribe(&mut self, callback: Box<dyn Fn()>) {
        self.subscribers.push(callback);
    }
}