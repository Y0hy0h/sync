mod utils;

use std::{convert::TryInto, rc::Rc, sync::RwLock};

use js_sys::{JsString, Promise};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::{Response, Storage};

use sync::{memory_db::MemoryDb as MDb, Db, FilePath};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct JsStore(Storage);

#[wasm_bindgen]
impl JsStore {
    #[wasm_bindgen(constructor)]
    pub fn new_store() -> JsStore {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        JsStore(storage)
    }

    #[wasm_bindgen]
    pub fn store(&mut self, key: &str, value: &str) {
        self.0.set_item(key, value).unwrap()
    }

    #[wasm_bindgen]
    pub fn read(&self, key: &str) -> Option<String> {
        self.0.get_item(key).unwrap()
    }
}

#[wasm_bindgen]
pub struct MemoryDb(Rc<RwLock<MDb<String>>>);

#[wasm_bindgen]
impl MemoryDb {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        MemoryDb(Rc::new(RwLock::new(MDb::new())))
    }

    #[wasm_bindgen]
    pub fn insert(&mut self, path: String, item: String) -> Promise {
        let path = vec![path].try_into().unwrap();
        let rc = self.0.clone();
        future_to_promise(async move {
            let mut store = rc.write().unwrap();
            store.insert_item(path, item).await;
            Ok(JsValue::UNDEFINED)
        })
    }

    #[wasm_bindgen]
    pub fn read(&self, path: String) -> Promise {
        let path = vec![path].try_into().unwrap();
        let rc = self.0.clone();
        future_to_promise(async move {
            let store = rc.read().unwrap();
            let result = store.get(&path).await.map(|s| s.to_string());
            Ok(result.into())
        })
    }
}

#[wasm_bindgen]
pub fn fetch_flowers() -> Promise {
    wasm_bindgen_futures::future_to_promise(fetch())
}

async fn fetch() -> Result<JsValue, JsValue> {
    let response_value = JsFuture::from(web_sys::window().ok_or_else(|| JsValue::from("No window."))?.fetch_with_str(
        "https://raw.githubusercontent.com/mdn/fetch-examples/master/basic-fetch/flowers.jpg",
    ))
    .await?;
    let response: Response = response_value.dyn_into()?;
    Ok(JsFuture::from(response.blob()?).await?)
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    web_sys::window()
        .unwrap()
        .alert_with_message(&format!("Hello, {}!", name))
        .unwrap();
}
