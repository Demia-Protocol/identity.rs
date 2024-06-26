// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use identity_iota::core::Context;
use identity_iota::core::Object;
use identity_iota::credential::JwtPresentation;
use identity_iota::credential::JwtPresentationBuilder;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::common::ArrayString;
use crate::common::MapStringAny;
use crate::credential::jwt_presentation::jwt_presentation_builder::IJwtPresentation;
use crate::credential::ArrayContext;
use crate::credential::ArrayJwt;
use crate::credential::ArrayPolicy;
use crate::credential::ArrayRefreshService;
use crate::credential::WasmJwt;
use crate::error::Result;
use crate::error::WasmResult;

#[wasm_bindgen(js_name = JwtPresentation, inspectable)]
pub struct WasmJwtPresentation(pub(crate) JwtPresentation);

#[wasm_bindgen(js_class = JwtPresentation)]
impl WasmJwtPresentation {
  /// Returns the base JSON-LD context.
  #[wasm_bindgen(js_name = "BaseContext")]
  pub fn base_context() -> Result<String> {
    match JwtPresentation::<Object>::base_context() {
      Context::Url(url) => Ok(url.to_string()),
      Context::Obj(_) => Err(JsError::new("JwtPresentation.BaseContext should be a single URL").into()),
    }
  }

  /// Returns the base type.
  #[wasm_bindgen(js_name = "BaseType")]
  pub fn base_type() -> String {
    JwtPresentation::<Object>::base_type().to_owned()
  }

  /// Constructs a new presentation.
  #[wasm_bindgen(constructor)]
  pub fn new(values: IJwtPresentation) -> Result<WasmJwtPresentation> {
    let builder: JwtPresentationBuilder = JwtPresentationBuilder::try_from(values)?;
    builder.build().map(Self).wasm_result()
  }

  /// Returns a copy of the JSON-LD context(s) applicable to the presentation.
  #[wasm_bindgen]
  pub fn context(&self) -> Result<ArrayContext> {
    self
      .0
      .context
      .iter()
      .map(JsValue::from_serde)
      .collect::<std::result::Result<js_sys::Array, _>>()
      .wasm_result()
      .map(|value| value.unchecked_into::<ArrayContext>())
  }

  /// Returns a copy of the unique `URI` identifying the presentation.
  #[wasm_bindgen]
  pub fn id(&self) -> Option<String> {
    self.0.id.as_ref().map(|url| url.to_string())
  }

  /// Returns a copy of the URIs defining the type of the presentation.
  #[wasm_bindgen(js_name = "type")]
  pub fn types(&self) -> ArrayString {
    self
      .0
      .types
      .iter()
      .map(|s| s.as_str())
      .map(JsValue::from_str)
      .collect::<js_sys::Array>()
      .unchecked_into::<ArrayString>()
  }

  /// Returns the JWT credentials expressing the claims of the presentation.
  #[wasm_bindgen(js_name = verifiableCredential)]
  pub fn verifiable_credential(&self) -> ArrayJwt {
    self
      .0
      .verifiable_credential
      .iter()
      .cloned()
      .map(WasmJwt::new)
      .map(JsValue::from)
      .collect::<js_sys::Array>()
      .unchecked_into::<ArrayJwt>()
  }

  /// Returns a copy of the URI of the entity that generated the presentation.
  #[wasm_bindgen]
  pub fn holder(&self) -> String {
    self.0.holder.as_ref().to_string()
  }

  /// Returns a copy of the service(s) used to refresh an expired {@link Credential} in the presentation.
  #[wasm_bindgen(js_name = "refreshService")]
  pub fn refresh_service(&self) -> Result<ArrayRefreshService> {
    self
      .0
      .refresh_service
      .iter()
      .map(JsValue::from_serde)
      .collect::<std::result::Result<js_sys::Array, _>>()
      .wasm_result()
      .map(|value| value.unchecked_into::<ArrayRefreshService>())
  }

  /// Returns a copy of the terms-of-use specified by the presentation holder
  #[wasm_bindgen(js_name = "termsOfUse")]
  pub fn terms_of_use(&self) -> Result<ArrayPolicy> {
    self
      .0
      .terms_of_use
      .iter()
      .map(JsValue::from_serde)
      .collect::<std::result::Result<js_sys::Array, _>>()
      .wasm_result()
      .map(|value| value.unchecked_into::<ArrayPolicy>())
  }

  /// Optional proof that can be verified by users in addition to JWS.
  #[wasm_bindgen]
  pub fn proof(&self) -> Result<Option<MapStringAny>> {
    self.0.proof.clone().map(MapStringAny::try_from).transpose()
  }

  /// Returns a copy of the miscellaneous properties on the presentation.
  #[wasm_bindgen]
  pub fn properties(&self) -> Result<MapStringAny> {
    MapStringAny::try_from(&self.0.properties)
  }
}

impl_wasm_json!(WasmJwtPresentation, JwtPresentation);
impl_wasm_clone!(WasmJwtPresentation, JwtPresentation);

impl From<JwtPresentation> for WasmJwtPresentation {
  fn from(presentation: JwtPresentation) -> WasmJwtPresentation {
    Self(presentation)
  }
}
