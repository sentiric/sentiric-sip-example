use super::types::ActiveCall;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Paylaşılan uygulama durumu. Aktif aramaları thread'ler arası güvenli bir şekilde tutar.
pub type AppState = Arc<Mutex<HashMap<String, ActiveCall>>>;