// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use identity_iota::account_storage::AgreementInfo;
use wasm_bindgen::prelude::*;

/// Agreement information used as the input for the concat KDF.
#[wasm_bindgen(js_name = AgreementInfo, inspectable)]
pub struct WasmAgreementInfo(pub(crate) AgreementInfo);

#[wasm_bindgen(js_class = AgreementInfo)]
impl WasmAgreementInfo {
  /// Creates an `AgreementInfo` Object.
  #[wasm_bindgen(constructor)]
  pub fn new(apu: Vec<u8>, apv: Vec<u8>, pub_info: Vec<u8>, priv_info: Vec<u8>) -> WasmAgreementInfo {
    WasmAgreementInfo(AgreementInfo::new(apu, apv, pub_info, priv_info))
  }

  /// Returns a copy of `apu'
  #[wasm_bindgen(js_name = apu)]
  pub fn apu(&self) -> Vec<u8> {
    self.0.apu.clone()
  }

  /// Returns a copy of `apv'
  #[wasm_bindgen(js_name = apv)]
  pub fn apv(&self) -> Vec<u8> {
    self.0.apv.clone()
  }

  /// Returns a copy of `pubInfo'
  #[wasm_bindgen(js_name = pubInfo)]
  pub fn pub_info(&self) -> Vec<u8> {
    self.0.pub_info.clone()
  }

  /// Returns a copy of `privInfo'
  #[wasm_bindgen(js_name = privInfo)]
  pub fn priv_info(&self) -> Vec<u8> {
    self.0.priv_info.clone()
  }
}

impl_wasm_json!(WasmAgreementInfo, AgreementInfo);

impl From<WasmAgreementInfo> for AgreementInfo {
  fn from(wasm_agreement_info: WasmAgreementInfo) -> Self {
    wasm_agreement_info.0
  }
}

impl From<AgreementInfo> for WasmAgreementInfo {
  fn from(agreement_info: AgreementInfo) -> Self {
    WasmAgreementInfo(agreement_info)
  }
}
