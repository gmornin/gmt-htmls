mod utils;

use js_sys::RegExp;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{RequestInit, UrlSearchParams, Document, HtmlInputElement, HtmlButtonElement, Headers};

const GMT: &str = " - GMT";

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
    update();
}

#[wasm_bindgen]
pub fn update() {
    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    let params_string = window.location().search().unwrap();
    let params =
        UrlSearchParams::new_with_str(&params_string).unwrap_or(UrlSearchParams::new().unwrap());
    let r#type = params.get("type").unwrap_or_default();

    let (hide, show, title) = match r#type.as_str() {
        "new" => ("signin", "signup", "Create account"),
        _ => ("signup", "signin", "Sign in"),
    };

    doc.set_title(&format!("{title}{GMT}"));

    let hide = doc.get_elements_by_class_name(hide);
    (0..hide.length()).for_each(|i| {
        let element = hide.get_with_index(i).unwrap();
        element.class_list().add_1("hide").unwrap();
    });

    let show = doc.get_elements_by_class_name(show);
    (0..show.length()).for_each(|i| {
        let element = show.get_with_index(i).unwrap();
        element.class_list().remove_1("hide").unwrap();
    });
}

#[wasm_bindgen]
pub fn change_state(t: &str) {
    let window = web_sys::window().unwrap();
    let history = window.history().unwrap();

    let new_url = format!(
        "{}{}",
        window.location().pathname().unwrap(),
        if t == "signup" { "?type=new" } else { "" }
    );

    history
        .push_state_with_url(&JsValue::UNDEFINED, "", Some(&new_url))
        .unwrap();

    update();
}

#[wasm_bindgen]
pub fn signin() {
    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    let identifier = get_value(&doc, "identifier");
    let password = get_value(&doc, "password");
    let error_display = doc.get_element_by_id("error-display").unwrap();

    let mut error = match (identifier.is_empty(), password.is_empty()) {
        (true, true) => Some("Identifier and password cannt be blank"),
        (true, false) => Some("Identifier cannot be blank"),
        (false, true) => Some("Password cannot be blank"),
        (false, false) => None,
    };

    let identifier_type = match (identifier.contains("@") ,RegExp::new(".+@.+\\..+", "").exec(&identifier).is_some()) {
        (_, true) => IdentifierType::Email,
        (true, false) => {
            error = Some("Invalid email address");
            IdentifierType::Username
        },
        _ => IdentifierType::Username,
    };

    error_display.set_text_content(error);
    if error.is_some() {
        error_display.class_list().remove_1("hide").unwrap();
        return;
    } else {
        error_display.class_list().add_1("hide").unwrap();
    }

    let req = GetToken {
        identifier,
        identifier_type,
        password,
    };

    disable_buttons(&doc, true);

    let ok = Closure::new(move |res| {
        web_sys::console::log_1(&res);
        disable_buttons(&doc, false);
    });

    let headers = Headers::new().unwrap();
    headers.append("content-type", "application/json").unwrap();

    web_sys::console::log_1(&serde_wasm_bindgen::to_value(&req).unwrap());

    let _ = window
        .fetch_with_str_and_init("/api/v1/account/gettoken", &RequestInit::new().method("POST").headers(&headers).body(Some(&serde_wasm_bindgen::to_value(&req).unwrap())))
        .then(&ok);

    ok.forget();
}


#[wasm_bindgen]
pub fn get_value(doc: &Document, id: &str) -> String {
    doc.get_element_by_id(id).unwrap().dyn_ref::<HtmlInputElement>().unwrap().value()
}

#[wasm_bindgen]
pub fn disable_buttons(doc: &Document, val: bool) {
    let signin = doc.get_element_by_id("submit-signin").unwrap();
    let signup = doc.get_element_by_id("submit-create").unwrap();
    signin.dyn_ref::<HtmlButtonElement>().unwrap().set_disabled(val);
    signup.dyn_ref::<HtmlButtonElement>().unwrap().set_disabled(val);
    
    if val {
        signin.set_text_content(Some("Signing in..."));
        signup.set_text_content(Some("Creating account..."))
    } else {
        signin.set_text_content(Some("Sign in"));
        signup.set_text_content(Some("Create account"))
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Responses {
    #[serde(rename = "error")]
    Error { kind: ErrorKind },
    #[serde(rename = "token")]
    GetToken { token: String },
}

#[derive(Deserialize)]
pub enum ErrorKind {
    #[serde(rename = "no such user")]
    NoSuchUser,
    #[serde(rename = "password incorrect")]
    PasswordIncorrect,
    #[serde(rename = "external")]
    External(String),
}

#[derive(Serialize)]
struct GetToken {
    pub identifier: String,
    pub identifier_type: IdentifierType,
    pub password: String,
}

#[derive(Serialize)]
pub enum IdentifierType {
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "id")]
    Id,
    #[serde(rename = "username")]
    Username,
}
