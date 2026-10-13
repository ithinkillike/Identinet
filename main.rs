//! Minimal NDN-inspired packet system in pure Rust.
//! Phase 1 starter: Name, Interest, Data, simple encode/decode, and a single-process demo.
//!
//! Intentionally simplified (not full NDN-TLV yet) 

use std::collections::VecDeque;
use std::fmt;

/// Hierarchical name (e.g. /edu/ucla/notes/v1)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    components: Vec<String>,
}

impl Name {
    pub fn new(components: Vec<String>) -> Self {
        Self { components }
    }

    pub fn from_uri(uri: &str) -> Self {
        let clean = uri.trim_start_matches('/');
        if clean.is_empty() {
            return Self { components: vec![] };
        }
        Self {
            components: clean.split('/').map(|s| s.to_string()).collect(),
        }
    }

    pub fn is_prefix_of(&self, other: &Name) -> bool {
        other.components.starts_with(&self.components)
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.components.is_empty() {
            write!(f, "/")
        } else {
            write!(f, "/{}", self.components.join("/"))
        }
    }
}

/// Interest packet (consumer request)
#[derive(Debug, Clone)]
pub struct Interest {
    pub name: Name,
    pub nonce: u32,
    pub lifetime_ms: u32,
    pub must_be_fresh: bool,
}

impl Interest {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            nonce: rand_nonce(),
            lifetime_ms: 4000,
            must_be_fresh: false,
        }
    }
}

/// Data packet (response)
#[derive(Debug, Clone)]
pub struct Data {
    pub name: Name,
    pub content: Vec<u8>,
    pub freshness_ms: u32,
}

impl Data {
    pub fn new(name: Name, content: impl Into<Vec<u8>>) -> Self {
        Self {
            name,
            content: content.into(),
            freshness_ms: 10_000,
        }
    }
}

/// Very simple length-prefixed encoding (not real NDN-TLV).
/// Format: [type:1][name_len:2][name_bytes][payload...]
/// This is just so you have something that round-trips. Replace with real TLV later.
pub fn encode_interest(interest: &Interest) -> Vec<u8> {
    let name_str = interest.name.to_string();
    let name_bytes = name_str.as_bytes();
    let mut buf = Vec::new();
    buf.push(0x05); // Interest type marker
    buf.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
    buf.extend_from_slice(name_bytes);
    buf.extend_from_slice(&interest.nonce.to_be_bytes());
    buf.extend_from_slice(&interest.lifetime_ms.to_be_bytes());
    buf.push(if interest.must_be_fresh { 1 } else { 0 });
    buf
}

pub fn decode_interest(bytes: &[u8]) -> Option<Interest> {
    if bytes.len() < 1 + 2 || bytes[0] != 0x05 {
        return None;
    }
    let name_len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
    if bytes.len() < 3 + name_len + 4 + 4 + 1 {
        return None;
    }
    let name_str = std::str::from_utf8(&bytes[3..3 + name_len]).ok()?;
    let name = Name::from_uri(name_str);
    let mut offset = 3 + name_len;
    let nonce = u32::from_be_bytes(bytes[offset..offset + 4].try_into().ok()?);
    offset += 4;
    let lifetime = u32::from_be_bytes(bytes[offset..offset + 4].try_into().ok()?);
    offset += 4;
    let must_be_fresh = bytes[offset] != 0;

    Some(Interest {
        name,
        nonce,
        lifetime_ms: lifetime,
        must_be_fresh,
    })
}

pub fn encode_data(data: &Data) -> Vec<u8> {
    let name_str = data.name.to_string();
    let name_bytes = name_str.as_bytes();
    let mut buf = Vec::new();
    buf.push(0x06); // Data type marker
    buf.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
    buf.extend_from_slice(name_bytes);
    buf.extend_from_slice(&(data.content.len() as u32).to_be_bytes());
    buf.extend_from_slice(&data.content);
    buf.extend_from_slice(&data.freshness_ms.to_be_bytes());
    buf
}

pub fn decode_data(bytes: &[u8]) -> Option<Data> {
    if bytes.len() < 1 + 2 || bytes[0] != 0x06 {
        return None;
    }
    let name_len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
    if bytes.len() < 3 + name_len + 4 {
        return None;
    }
    let name_str = std::str::from_utf8(&bytes[3..3 + name_len]).ok()?;
    let name = Name::from_uri(name_str);
    let mut offset = 3 + name_len;
    let content_len = u32::from_be_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if bytes.len() < offset + content_len + 4 {
        return None;
    }
    let content = bytes[offset..offset + content_len].to_vec();
    offset += content_len;
    let freshness = u32::from_be_bytes(bytes[offset..offset + 4].try_into().ok()?);

    Some(Data {
        name,
        content,
        freshness_ms: freshness,
    })
}

/// Tiny helper for nonces (not cryptographically secure — fine for starter)
fn rand_nonce() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

/// Very simple in-memory Content Store for the demo
struct SimpleCs {
    store: VecDeque<(Name, Data)>,
}

impl SimpleCs {
    fn new() -> Self {
        Self {
            store: VecDeque::new(),
        }
    }

    fn insert(&mut self, data: Data) {
        self.store.push_back((data.name.clone(), data));
        // Keep it small
        if self.store.len() > 20 {
            self.store.pop_front();
        }
    }

    fn lookup(&self, interest: &Interest) -> Option<&Data> {
        for (name, data) in &self.store {
            if name == &interest.name || interest.name.is_prefix_of(name) {
                return Some(data);
            }
        }
        None
    }
}

fn main() {
    println!("=== Phase 1: Minimal NDN Packet System Demo ===\n");

    // 1. Create a name and Interest
    let name = Name::from_uri("/demo/hello/world");
    let interest = Interest::new(name.clone());
    println!("Created Interest for: {}", interest.name);
    println!("Nonce: {}", interest.nonce);

    // 2. Encode → Decode round-trip
    let wire = encode_interest(&interest);
    println!("Encoded Interest length: {} bytes", wire.len());
    let decoded = decode_interest(&wire).expect("decode failed");
    assert_eq!(decoded.name, interest.name);
    println!("Decode round-trip OK\n");

    // 3. Create matching Data and put it in a tiny CS
    let data = Data::new(name.clone(), b"Hello from customized NDN!".to_vec());
    let mut cs = SimpleCs::new();
    cs.insert(data.clone());

    // 4. Simulate lookup
    if let Some(found) = cs.lookup(&interest) {
        println!("CS hit! Content: {}", String::from_utf8_lossy(&found.content));
        let data_wire = encode_data(found);
        println!("Encoded Data length: {} bytes", data_wire.len());
        let decoded_data = decode_data(&data_wire).expect("data decode failed");
        println!("Data decode OK: {}", String::from_utf8_lossy(&decoded_data.content));
    } else {
        println!("CS miss");
    }

    println!("\n--- Next steps ---");
    println!("1. cargo run   (you just did)");
    println!("2. Move Name / Interest / Data into separate modules");
    println!("3. Add a proper PIT and FIB");
    println!("4. Replace the simple encoding with real NDN-TLV");
    println!("5. Add faces (UDP or channels) so two processes can talk");
}
