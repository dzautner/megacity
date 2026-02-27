//! IndexedDB storage backend for WASM saves.
//!
//! Replaces the old localStorage + base64 approach with IndexedDB which
//! supports storing large binary blobs (50 MB+) without the ~5 MB
//! localStorage limit.

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    DomException, IdbDatabase, IdbObjectStore, IdbOpenDbRequest, IdbRequest, IdbTransactionMode,
    Window,
};

const DB_NAME: &str = "megacity_saves";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "saves";
const SAVE_KEY: &str = "megacity_save";

/// Legacy localStorage key (for migration).
const LEGACY_KEY: &str = "megacity_save";

/// Error type for WASM save/load operations.
#[derive(Debug, Clone)]
pub enum WasmStorageError {
    /// The browser storage quota has been exceeded.
    QuotaExceeded,
    /// A general storage error with a descriptive message.
    Other(String),
}

impl std::fmt::Display for WasmStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmStorageError::QuotaExceeded => {
                write!(
                    f,
                    "Save failed: storage full. Try deleting old saves or clearing browser data."
                )
            }
            WasmStorageError::Other(msg) => write!(f, "Save failed: {}", msg),
        }
    }
}

/// Check whether a JsValue represents a QuotaExceededError from IndexedDB.
fn is_quota_exceeded_error(err: &JsValue) -> bool {
    // Check if it's a DOMException with name "QuotaExceededError"
    if let Ok(dom_exception) = err.clone().dyn_into::<DomException>() {
        return dom_exception.name() == "QuotaExceededError";
    }
    // Also check the string representation as a fallback
    let s = format!("{:?}", err);
    s.contains("QuotaExceededError") || s.contains("quota")
}

fn window() -> Result<Window, WasmStorageError> {
    web_sys::window().ok_or_else(|| WasmStorageError::Other("no window".to_string()))
}

/// Open (or create) the IndexedDB database.
async fn open_db() -> Result<IdbDatabase, WasmStorageError> {
    let window = window()?;
    let idb_factory = window
        .indexed_db()
        .map_err(|e| WasmStorageError::Other(format!("indexedDB error: {:?}", e)))?
        .ok_or(WasmStorageError::Other(
            "indexedDB not available".to_string(),
        ))?;

    let open_request: IdbOpenDbRequest = idb_factory
        .open_with_u32(DB_NAME, DB_VERSION)
        .map_err(|e| WasmStorageError::Other(format!("failed to open db: {:?}", e)))?;

    // Handle upgrade (create object store on first use).
    let on_upgrade = Closure::once(move |event: web_sys::IdbVersionChangeEvent| {
        let Some(target) = event.target() else {
            return;
        };
        let request: IdbOpenDbRequest = target.unchecked_into();
        let Ok(result) = request.result() else {
            return;
        };
        let db: IdbDatabase = result.unchecked_into();
        if !db.object_store_names().contains(STORE_NAME) {
            let _ = db.create_object_store(STORE_NAME);
        }
    });
    open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));

    let db = idb_request_to_future(&open_request).await?;
    let db: IdbDatabase = db.unchecked_into();

    // Drop closure so it doesn't leak
    drop(on_upgrade);

    Ok(db)
}

/// Convert an IdbRequest into a Future that resolves when the request completes.
async fn idb_request_to_future(request: &IdbRequest) -> Result<JsValue, WasmStorageError> {
    let (sender, receiver) =
        futures_channel::oneshot::channel::<Result<JsValue, WasmStorageError>>();
    let sender = std::rc::Rc::new(std::cell::RefCell::new(Some(sender)));

    let sender_ok = sender.clone();
    let request_clone = request.clone();
    let on_success = Closure::once(move |_event: web_sys::Event| {
        if let Some(tx) = sender_ok.borrow_mut().take() {
            let result = request_clone.result().unwrap_or(JsValue::UNDEFINED);
            let _ = tx.send(Ok(result));
        }
    });

    let sender_err = sender;
    let request_clone2 = request.clone();
    let on_error = Closure::once(move |_event: web_sys::Event| {
        if let Some(tx) = sender_err.borrow_mut().take() {
            let err_val = request_clone2.error().ok().flatten();
            let error = if let Some(ref dom_exc) = err_val {
                let js: &JsValue = dom_exc.as_ref();
                if is_quota_exceeded_error(js) {
                    WasmStorageError::QuotaExceeded
                } else {
                    WasmStorageError::Other(format!("{:?}", dom_exc.message()))
                }
            } else {
                WasmStorageError::Other("unknown IDB error".to_string())
            };
            let _ = tx.send(Err(error));
        }
    });

    request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
    request.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    let result = receiver
        .await
        .map_err(|_| WasmStorageError::Other("IDB request channel dropped".to_string()))?;

    // Clean up event handlers
    request.set_onsuccess(None);
    request.set_onerror(None);

    result
}

/// Save compressed binary data to IndexedDB.
pub async fn idb_save(bytes: Vec<u8>) -> Result<(), WasmStorageError> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    // Compress with deflate (same as before, but no base64 step).
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&bytes)
        .map_err(|e| WasmStorageError::Other(format!("compression write error: {}", e)))?;
    let compressed = encoder
        .finish()
        .map_err(|e| WasmStorageError::Other(format!("compression finish error: {}", e)))?;

    let db = open_db().await?;

    let transaction = db
        .transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)
        .map_err(|e| WasmStorageError::Other(format!("transaction error: {:?}", e)))?;
    let store: IdbObjectStore = transaction
        .object_store(STORE_NAME)
        .map_err(|e| WasmStorageError::Other(format!("object store error: {:?}", e)))?;

    // Store as Uint8Array (binary, no base64 overhead).
    let js_array = Uint8Array::from(&compressed[..]);
    let put_result = store.put_with_key(&js_array, &JsValue::from_str(SAVE_KEY));

    let request = match put_result {
        Ok(req) => req,
        Err(e) => {
            if is_quota_exceeded_error(&e) {
                return Err(WasmStorageError::QuotaExceeded);
            }
            return Err(WasmStorageError::Other(format!("put error: {:?}", e)));
        }
    };

    idb_request_to_future(&request).await?;

    web_sys::console::log_1(
        &format!(
            "Saved {} bytes ({} compressed) to IndexedDB",
            bytes.len(),
            compressed.len()
        )
        .into(),
    );

    Ok(())
}

/// Load compressed binary data from IndexedDB.
/// Falls back to localStorage for migration of old saves.
pub async fn idb_load() -> Result<Vec<u8>, String> {
    // First try IndexedDB.
    match idb_load_from_db().await {
        Ok(Some(bytes)) => return Ok(bytes),
        Ok(None) => {
            // No save in IndexedDB; try migrating from localStorage.
            web_sys::console::log_1(
                &"No save in IndexedDB, checking localStorage for migration...".into(),
            );
        }
        Err(e) => {
            web_sys::console::log_1(
                &format!(
                    "IndexedDB load failed ({}), trying localStorage fallback...",
                    e
                )
                .into(),
            );
        }
    }

    // Try loading from legacy localStorage.
    match load_from_local_storage() {
        Ok(bytes) => {
            web_sys::console::log_1(&"Migrated save from localStorage to IndexedDB".into());
            // Migrate: save to IndexedDB so future loads use IndexedDB directly.
            // We re-compress inside idb_save, so pass the decompressed bytes.
            let bytes_clone = bytes.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = idb_save(bytes_clone).await {
                    web_sys::console::log_1(
                        &format!("Migration save to IndexedDB failed: {}", e).into(),
                    );
                } else {
                    // Remove legacy localStorage entry after successful migration.
                    let _ = remove_legacy_local_storage();
                }
            });
            Ok(bytes)
        }
        Err(_) => Err("no save found".to_string()),
    }
}

/// Attempt to load from IndexedDB. Returns Ok(None) if no save exists.
async fn idb_load_from_db() -> Result<Option<Vec<u8>>, String> {
    use flate2::read::DeflateDecoder;
    use std::io::Read;

    let db = open_db().await.map_err(|e| e.to_string())?;

    let transaction = db
        .transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readonly)
        .map_err(|e| format!("transaction error: {:?}", e))?;
    let store = transaction
        .object_store(STORE_NAME)
        .map_err(|e| format!("object store error: {:?}", e))?;

    let request = store
        .get(&JsValue::from_str(SAVE_KEY))
        .map_err(|e| format!("get error: {:?}", e))?;

    let result = idb_request_to_future(&request)
        .await
        .map_err(|e| e.to_string())?;

    if result.is_undefined() || result.is_null() {
        return Ok(None);
    }

    let array: Uint8Array = result.unchecked_into();
    let raw = array.to_vec();

    // Decompress; fall back to raw for uncompressed data.
    let mut decoder = DeflateDecoder::new(&raw[..]);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(Some(decompressed)),
        Err(_) => Ok(Some(raw)),
    }
}

/// Load from legacy localStorage (base64-encoded, possibly compressed).
fn load_from_local_storage() -> Result<Vec<u8>, String> {
    use flate2::read::DeflateDecoder;
    use std::io::Read;

    let window = window().map_err(|e| e.to_string())?;
    let storage = window
        .local_storage()
        .map_err(|_| "localStorage error")?
        .ok_or("no localStorage")?;
    let encoded = storage
        .get_item(LEGACY_KEY)
        .map_err(|_| "failed to get localStorage item")?
        .ok_or("no save found in localStorage")?;
    let raw = base64_decode(&encoded).map_err(|e| format!("base64 decode error: {}", e))?;

    // Try to decompress; fall back to raw bytes for old uncompressed saves.
    let mut decoder = DeflateDecoder::new(&raw[..]);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(_) => Ok(raw),
    }
}

/// Remove legacy localStorage entry after migration.
fn remove_legacy_local_storage() -> Result<(), String> {
    let window = window().map_err(|e| e.to_string())?;
    let storage = window
        .local_storage()
        .map_err(|_| "localStorage error")?
        .ok_or("no localStorage")?;
    storage
        .remove_item(LEGACY_KEY)
        .map_err(|_| "failed to remove localStorage item")?;
    Ok(())
}

/// Decode base64 string to bytes (for legacy localStorage migration).
fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    fn decode_char(c: u8) -> Result<u32, &'static str> {
        match c {
            b'A'..=b'Z' => Ok((c - b'A') as u32),
            b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err("invalid base64 character"),
        }
    }
    let input = input.as_bytes();
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let chunks: Vec<&[u8]> = input.chunks(4).collect();
    for chunk in chunks {
        if chunk.len() < 2 {
            break;
        }
        let a = decode_char(chunk[0])?;
        let b = decode_char(chunk[1])?;
        result.push(((a << 2) | (b >> 4)) as u8);
        if chunk.len() > 2 && chunk[2] != b'=' {
            let c = decode_char(chunk[2])?;
            result.push((((b & 0xF) << 4) | (c >> 2)) as u8);
            if chunk.len() > 3 && chunk[3] != b'=' {
                let d = decode_char(chunk[3])?;
                result.push((((c & 0x3) << 6) | d) as u8);
            }
        }
    }
    Ok(result)
}
