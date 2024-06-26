// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::convert::TryFrom;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::str::FromStr;

use identity_core::common::KeyComparable;
use identity_did::BaseDIDUrl;
use identity_did::CoreDID;
use identity_did::Error as DIDError;
use identity_did::DID;
use isocountry::CountryCode;
use ref_cast::ref_cast_custom;
use ref_cast::RefCastCustom;
use serde::Deserialize;
use serde::Serialize;

use crate::NetworkName;

pub type Result<T> = std::result::Result<T, DIDError>;

/// A DID conforming to the IOTA DID method specification.
///
/// This is a thin wrapper around the [`DID`][`CoreDID`] type from the
/// [`identity_did`][`identity_did`] crate.
#[derive(Clone, Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, RefCastCustom)]
#[repr(transparent)]
#[serde(into = "CoreDID", try_from = "CoreDID")]
pub struct DemiaDID(CoreDID);

impl DemiaDID {
  /// The URL scheme for Decentralized Identifiers.
  pub const SCHEME: &'static str = CoreDID::SCHEME;

  /// The Demia DID method name (`"demia"`).
  pub const METHOD: &'static str = "demia";

  /// The default network name (`"dmia"`).
  pub const DEFAULT_NETWORK: &'static str = "dmia";

  /// The default country code (`"usa"`).
  pub const DEFAULT_COUNTRY: &'static str = "usa";

  /// The tag of the placeholder DID.
  pub const PLACEHOLDER_TAG: &'static str = "0x0000000000000000000000000000000000000000000000000000000000000000";

  /// The length of an Alias ID, which is a BLAKE2b-256 hash (32-bytes).
  pub(crate) const TAG_BYTES_LEN: usize = 32;

  /// Convert a `CoreDID` reference to an `DemiaDID` reference without checking the referenced value.
  ///  
  /// # Warning
  /// This method should only be called on [`CoreDIDs`](CoreDID) that
  /// are known to satisfy the requirements of the IOTA UTXO specification.  
  ///
  /// # Memory safety
  ///
  /// The `ref-cast` crate ensures a memory safe implementation.  
  #[ref_cast_custom]
  pub(crate) const fn from_inner_ref_unchecked(did: &CoreDID) -> &Self;

  // ===========================================================================
  // Constructors
  // ===========================================================================

  /// Constructs a new [`DemiaDID`] from a byte representation of the tag and the given
  /// network name.
  ///
  /// See also [`DemiaDID::placeholder`].
  ///
  /// # Example
  ///
  /// ```
  /// # use identity_did::DID;
  /// # use identity_demia_core::NetworkName;
  /// # use identity_demia_core::DemiaDID;
  /// # use isocountry::CountryCode;
  /// #
  /// let did = DemiaDID::new(&[1;32], &CountryCode::USA, &NetworkName::try_from("dmia").unwrap());
  /// assert_eq!(did.as_str(), "did:demia:0x0101010101010101010101010101010101010101010101010101010101010101");
  pub fn new(bytes: &[u8; 32], country_code: &CountryCode, network_name: &NetworkName) -> Self {
    let tag = prefix_hex::encode(bytes);
    let did: String = format!("did:{}:{}:{}:{}", Self::METHOD, country_code.alpha3().to_ascii_lowercase(), network_name, tag);

    Self::parse(did).expect("DIDs constructed with new should be valid")
  }

  /// Constructs a new [`DemiaDID`] from a hex representation of an Alias Id and the given
  /// network name.
  pub fn from_alias_id(alias_id: &str, country_code: &CountryCode, network_name: &NetworkName) -> Self {
    let did: String = format!("did:{}:{}:{}:{}", Self::METHOD, country_code.alpha3(), network_name, alias_id);
    Self::parse(did).expect("DIDs constructed with new should be valid")
  }

  /// Creates a new placeholder [`DemiaDID`] with the given network name.
  ///
  /// # Example
  ///
  /// ```
  /// # use identity_did::DID;
  /// # use identity_demia_core::NetworkName;
  /// # use identity_demia_core::DemiaDID;
  /// # use isocountry::CountryCode;
  /// #
  /// let placeholder = DemiaDID::placeholder(&CountryCode::USA, &NetworkName::try_from("dmia").unwrap());
  /// assert_eq!(placeholder.as_str(), "did:demia:0x0000000000000000000000000000000000000000000000000000000000000000");
  /// assert!(placeholder.is_placeholder());
  pub fn placeholder( country_code: &CountryCode, network_name: &NetworkName) -> Self {
    Self::new(&[0; 32], country_code, network_name)
  }

  /// Returns whether this is the placeholder DID.
  ///
  /// # Example
  ///
  /// ```
  /// # use identity_did::DID;
  /// # use identity_demia_core::NetworkName;
  /// # use identity_demia_core::DemiaDID;
  /// # use isocountry::CountryCode;
  /// #
  /// let placeholder = DemiaDID::placeholder(&CountryCode::USA, &NetworkName::try_from("dmia").unwrap());
  /// assert!(placeholder.is_placeholder());
  pub fn is_placeholder(&self) -> bool {
    self.tag() == Self::PLACEHOLDER_TAG
  }

  /// Parses an [`DemiaDID`] from the given `input`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the input does not conform to the [`DemiaDID`] specification.
  pub fn parse(input: impl AsRef<str>) -> Result<Self> {
    CoreDID::parse(input.as_ref().to_lowercase()).and_then(Self::try_from_core)
  }

  /// Converts a [`CoreDID`] to a [`DemiaDID`].
  ///
  /// # Errors
  ///
  /// Returns `Err` if the input does not conform to the [`DemiaDID`] specification.
  pub fn try_from_core(did: CoreDID) -> Result<Self> {
    Self::check_validity(&did)?;

    Ok(Self(Self::normalize(did)))
  }

  // ===========================================================================
  // Properties
  // ===========================================================================

  /// Returns the country name of the `DID`.
  pub fn country_str(&self) -> &str {
    Self::denormalized_components(self.method_id()).0
  }

  /// Returns the IOTA `network` name of the `DID`.
  pub fn network_str(&self) -> &str {
    Self::denormalized_components(self.method_id()).1
  }

  /// Returns the tag of the `DID`, which is a hex-encoded Alias ID.
  pub fn tag(&self) -> &str {
    Self::denormalized_components(self.method_id()).2
  }

  // ===========================================================================
  // Validation
  // ===========================================================================

  /// Checks if the given `DID` is syntactically valid according to the [`DemiaDID`] method specification.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the input is not a syntactically valid [`DemiaDID`].
  pub fn check_validity<D: DID>(did: &D) -> Result<()> {
    Self::check_method(did)
      .and_then(|_| Self::check_country(did))
      .and_then(|_| Self::check_network(did))
      .and_then(|_| Self::check_tag(did))
  }

  /// Returns a `bool` indicating if the given `DID` is valid according to the
  /// [`DemiaDID`] method specification.
  ///
  /// Equivalent to `DemiaDID::check_validity(did).is_ok()`.
  pub fn is_valid(did: &CoreDID) -> bool {
    Self::check_validity(did).is_ok()
  }

  // ===========================================================================
  // Helpers
  // ===========================================================================

  /// Checks if the given `DID` has a valid [`DemiaDID`] `method` (i.e. `"iota"`).
  ///
  /// # Errors
  ///
  /// Returns `Err` if the input represents another method.
  fn check_method<D: DID>(did: &D) -> Result<()> {
    (did.method() == Self::METHOD)
      .then_some(())
      .ok_or(DIDError::InvalidMethodName)
  }

  /// Checks if the given `DID` has a valid [`DemiaDID`] `method_id`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the input does not have a [`DemiaDID`] compliant method id.
  fn check_tag<D: DID>(did: &D) -> Result<()> {
    let (_, _, tag) = Self::denormalized_components(did.method_id());

    // Implicitly catches if there are too many segments (:) in the DID too.
    prefix_hex::decode::<[u8; Self::TAG_BYTES_LEN]>(tag)
      .map_err(|_| DIDError::InvalidMethodId)
      .map(|_| ())
  }

  /// Checks if the given `DID` has a valid [`DemiaDID`] country code.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the input is not a valid country code according to the ISO country alpha3 method specification.
  fn check_country<D: DID>(did: &D) -> Result<()> {
    let (country_code, _, _) = Self::denormalized_components(did.method_id());
    CountryCode::for_alpha3_caseless(country_code).map_err(|_| DIDError::Other("invalid country code"))?;
    Ok(())
  }

  /// Checks if the given `DID` has a valid [`DemiaDID`] network name.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the input is not a valid network name according to the [`DemiaDID`] method specification.
  fn check_network<D: DID>(did: &D) -> Result<()> {
    let (_, network_name, _) = Self::denormalized_components(did.method_id());
    NetworkName::validate_network_name(network_name).map_err(|_| DIDError::Other("invalid network name"))
  }

  /// Normalizes the DID `method_id` by removing the default network segment if present.
  ///
  /// E.g.
  /// - `"did:iota:main:123" -> "did:iota:123"` is normalized
  /// - `"did:iota:dev:123" -> "did:iota:dev:123"` is unchanged
  // TODO: Remove the lint once this bug in clippy has been fixed. Without to_owned a mutable reference will be aliased.
  #[allow(clippy::unnecessary_to_owned)]
  fn normalize(mut did: CoreDID) -> CoreDID {
    let method_id = did.method_id();
    let (country, network, tag) = Self::denormalized_components(method_id);
    if tag.len() == method_id.len() || network != Self::DEFAULT_NETWORK {
      did
    } else {
      did
        .set_method_id(tag.to_owned())
        .expect("normalizing a valid CoreDID should be Ok");
      did
    }
  }

  /// foo:bar -> (foo, DemiaDID::DEFAULT_NETWORK, bar)
  /// foo:bar:baz -> (foo, bar, baz)
  /// foo:bar:baz:rest -> (foo, bar, baz:rest)
  /// foo -> (DemiaDID::DEFAULT_COUNTRY, DemiaDID::DEFAULT_NETWORK.as_ref(), foo)
  #[inline(always)]
  fn denormalized_components(input: &str) -> (&str, &str, &str) {
    match input
      .find(':') {
        Some(idx) => {
          let (country, input) = input.split_at(idx);
          let rest = input[1..].find(':')
            .map(|idx| input[1..].split_at(idx))
            .map(|(network, tail)| (network, &tail[1..]))
            // Self::DEFAULT_NETWORK is built from a static reference so unwrapping is fine
            .unwrap_or((Self::DEFAULT_NETWORK, input));
          (country, rest.0, rest.1)
        },
        None => (Self::DEFAULT_COUNTRY, Self::DEFAULT_NETWORK, input)
      }
  }
}

impl FromStr for DemiaDID {
  type Err = DIDError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl TryFrom<&str> for DemiaDID {
  type Error = DIDError;

  fn try_from(other: &str) -> std::result::Result<Self, Self::Error> {
    Self::parse(other)
  }
}

impl TryFrom<String> for DemiaDID {
  type Error = DIDError;

  fn try_from(other: String) -> std::result::Result<Self, Self::Error> {
    Self::parse(other)
  }
}

impl Display for DemiaDID {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<DemiaDID> for CoreDID {
  fn from(id: DemiaDID) -> Self {
    id.0
  }
}

impl From<DemiaDID> for String {
  fn from(did: DemiaDID) -> Self {
    did.into_string()
  }
}

impl TryFrom<CoreDID> for DemiaDID {
  type Error = DIDError;
  fn try_from(value: CoreDID) -> std::result::Result<Self, Self::Error> {
    Self::try_from_core(value)
  }
}

impl TryFrom<BaseDIDUrl> for DemiaDID {
  type Error = DIDError;

  fn try_from(other: BaseDIDUrl) -> Result<Self> {
    let core_did: CoreDID = CoreDID::try_from(other)?;
    Self::try_from(core_did)
  }
}

impl AsRef<CoreDID> for DemiaDID {
  fn as_ref(&self) -> &CoreDID {
    &self.0
  }
}

impl KeyComparable for DemiaDID {
  type Key = CoreDID;

  #[inline]
  fn key(&self) -> &Self::Key {
    self.as_ref()
  }
}

#[cfg(feature = "client")]
mod __iota_did_client {
  use crate::block::output::AliasId;
  use crate::DemiaDID;

  impl From<&DemiaDID> for AliasId {
    /// Creates an [`AliasId`] from the DID tag.
    fn from(did: &DemiaDID) -> Self {
      let tag_bytes: [u8; DemiaDID::TAG_BYTES_LEN] = prefix_hex::decode(did.tag())
        .expect("being able to successfully decode the tag should be checked during DID creation");
      AliasId::new(tag_bytes)
    }
  }
}

#[cfg(test)]
mod tests {
  use identity_did::DIDUrl;
  use once_cell::sync::Lazy;
  use proptest::strategy::Strategy;
  use proptest::*;

  use super::*;

  // ===========================================================================================================================
  // Reusable constants and statics
  // ===========================================================================================================================

  // obtained AliasID from a valid OutputID string
  // output_id copied from https://github.com/iotaledger/bee/blob/30cab4f02e9f5d72ffe137fd9eb09723b4f0fdb6/bee-block/tests/output_id.rs
  // value of AliasID computed from AliasId::from(OutputId).to_string()
  const VALID_ALIAS_ID_STR: &str = "0xf29dd16310c2100fd1bf568b345fb1cc14d71caa3bd9b5ad735d2bd6d455ca3b";

  const LEN_VALID_ALIAS_STR: usize = VALID_ALIAS_ID_STR.len();

  static VALID_IOTA_DID_STRING: Lazy<String> = Lazy::new(|| format!("did:{}:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), VALID_ALIAS_ID_STR));

  // Rules are: at least one character, at most six characters and may only contain digits and/or lowercase ascii
  // characters.
  const VALID_NETWORK_NAMES: [&str; 13] = [
    DemiaDID::DEFAULT_NETWORK,
    "main",
    "dev",
    "smr",
    "rms",
    "test",
    "foo",
    "foobar",
    "123456",
    "0",
    "foo42",
    "bar123",
    "42foo",
  ];

  static VALID_IOTA_DID_STRINGS: Lazy<Vec<String>> = Lazy::new(|| {
    let network_tag_to_did = |network, tag| format!("did:{}:{}:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), network, tag);

    let valid_strings: Vec<String> = VALID_NETWORK_NAMES
      .iter()
      .flat_map(|network| {
        [VALID_ALIAS_ID_STR, DemiaDID::PLACEHOLDER_TAG]
          .iter()
          .map(move |tag| network_tag_to_did(network, tag))
      })
      .collect();

    // in principle the previous binding is not necessary (we could have just returned the value),
    // but let's just ensure that it contains the expected number of elements first.
    assert_eq!(valid_strings.len(), 2 * VALID_NETWORK_NAMES.len());

    valid_strings
  });

  // ===========================================================================================================================
  // Test check_* methods
  // ===========================================================================================================================

  #[test]
  fn invalid_check_method() {
    let did_key_core: CoreDID = CoreDID::parse("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").unwrap();
    assert!(matches!(
      DemiaDID::check_method(&did_key_core),
      Err(DIDError::InvalidMethodName)
    ));
  }

  #[test]
  fn valid_check_method() {
    let did_iota_core: CoreDID = CoreDID::parse(&*VALID_IOTA_DID_STRING).unwrap();
    assert!(DemiaDID::check_method(&did_iota_core).is_ok());
  }

  #[test]
  fn valid_check_network() {
    let assert_check_network = |input: &str| {
      let did_core: CoreDID =
        CoreDID::parse(input).unwrap_or_else(|_| panic!("expected {input} to parse to a valid CoreDID"));
      assert!(
        DemiaDID::check_network(&did_core).is_ok(),
        "test: valid_check_network failed with input {input}",
      );
    };

    for network_name in VALID_NETWORK_NAMES {
      let did_string = format!("did:method:{network_name}:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
      assert_check_network(&did_string);
    }

    assert_check_network("did:method:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
  }

  #[test]
  fn invalid_check_network() {
    // Loop over list of network names known to be invalid, attempt to create a CoreDID containing the given network
    // name in the method_id sub-string and ensure that `DemiaDID::check_network` fails. If the provided network
    // name is in conflict with the DID Core spec itself then proceed to the next network name.

    // Ensure that this test is robust to changes in the supplied list of network names, i.e. fail if none of the
    // network names can be contained in a generic CoreDID.

    let mut check_network_executed: bool = false;

    const INVALID_NETWORK_NAMES: [&str; 10] = [
      "Main", "fOo", "deV", "féta", "", "  ", "foo ", " foo", "1234567", "foobar0",
    ];
    for network_name in INVALID_NETWORK_NAMES {
      let did_string: String = format!("did:method:usa:{network_name}:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
      let did_core: CoreDID = {
        match CoreDID::parse(did_string) {
          Ok(did_core) => did_core,
          Err(_) => continue,
        }
      };

      assert!(matches!(DemiaDID::check_network(&did_core), Err(DIDError::Other(_))));
      check_network_executed = true;
    }
    assert!(
      check_network_executed,
      "DemiaDID::check_network` should have been executed"
    );
  }

  #[test]
  fn valid_check_tag() {
    for input in VALID_IOTA_DID_STRINGS.iter() {
      let did_core: CoreDID = CoreDID::parse(input).unwrap();
      assert!(
        DemiaDID::check_tag(&did_core).is_ok(),
        "test: valid_check_method_id failed on input {input}"
      );
    }

    // Should also work for DID's of the form: did:<method_name>:<valid_iota_network (or
    // nothing/normalized)>:<alias_id>
    let did_other_string: String = format!("did:method:{VALID_ALIAS_ID_STR}");
    let did_other_with_network: String = format!("did:method:usa:test:{VALID_ALIAS_ID_STR}");
    let did_other_core: CoreDID = CoreDID::parse(did_other_string).unwrap();
    let did_other_with_network_core: CoreDID = CoreDID::parse(did_other_with_network).unwrap();

    assert!(DemiaDID::check_tag(&did_other_core).is_ok());
    assert!(DemiaDID::check_tag(&did_other_with_network_core).is_ok());
  }

  #[test]
  fn invalid_check_tag() {
    let invalid_method_id_strings = [
      // Too many segments
      format!("did:method:usa:main:test:{VALID_ALIAS_ID_STR}"),
      // Tag is not prefixed
      format!("did:method:{}", &VALID_ALIAS_ID_STR.strip_prefix("0x").unwrap()),
      // Tag is too long 
      format!(
        "did:method:{}",
        &VALID_ALIAS_ID_STR.chars().chain("a".chars()).collect::<String>()
      ),
      // Tag is too short (omit last character)
      format!("did:method:main:{}", &VALID_ALIAS_ID_STR[..65]),
    ];

    for input in invalid_method_id_strings {
      let did_core: CoreDID = CoreDID::parse(input).unwrap();
      assert!(
        matches!(DemiaDID::check_tag(&did_core), Err(DIDError::InvalidMethodId)),
        "{}",
        did_core
      );
    }
  }

  // ===========================================================================================================================
  // Test constructors
  // ===========================================================================================================================

  #[test]
  fn placeholder_produces_a_did_with_expected_string_representation() {
    assert_eq!(
      DemiaDID::placeholder(&CountryCode::for_alpha3_caseless(DemiaDID::DEFAULT_COUNTRY).unwrap(), &NetworkName::try_from(DemiaDID::DEFAULT_NETWORK).unwrap()).as_str(),
      format!("did:{}:{}", DemiaDID::METHOD, DemiaDID::PLACEHOLDER_TAG)
    );

    for name in VALID_NETWORK_NAMES
      .iter()
      .filter(|name| *name != &DemiaDID::DEFAULT_NETWORK)
    {
      let network_name: NetworkName = NetworkName::try_from(*name).unwrap();
      let did: DemiaDID = DemiaDID::placeholder(&CountryCode::USA, &network_name);
      assert_eq!(
        did.as_str(),
        format!("did:{}:{}:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), name, DemiaDID::PLACEHOLDER_TAG)
      );
    }
  }

  #[test]
  fn normalization_in_constructors() {
    let did_with_default_network_string: String = format!(
      "did:{}:{}:{}:{}",
      DemiaDID::METHOD,
      DemiaDID::DEFAULT_COUNTRY,
      DemiaDID::DEFAULT_NETWORK,
      VALID_ALIAS_ID_STR
    );
    let expected_normalization_string_representation: String =
      format!("did:{}:{}", DemiaDID::METHOD, VALID_ALIAS_ID_STR);

    assert_eq!(
      DemiaDID::parse(did_with_default_network_string).unwrap().as_str(),
      expected_normalization_string_representation
    );
  }

  #[test]
  fn parse_valid() {
    for did_str in VALID_IOTA_DID_STRINGS.iter() {
      assert!(DemiaDID::parse(did_str).is_ok());
    }
  }

  #[test]
  fn parse_invalid() {
    let execute_assertions = |valid_alias_id: &str| {
      assert!(matches!(
        DemiaDID::parse(format!("dod:{}:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)),
        Err(DIDError::InvalidScheme)
      ));

      assert!(matches!(
        DemiaDID::parse(format!("did:key:{valid_alias_id}")),
        Err(DIDError::InvalidMethodName)
      ));

      // invalid network name (exceeded six characters)
      assert!(matches!(
        DemiaDID::parse(format!("did:{}:1234567:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)),
        Err(DIDError::Other(_))
      ));

      // invalid network name (contains non ascii character é)
      assert!(matches!(
        DemiaDID::parse(format!("did:{}:féta:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)),
        Err(DIDError::InvalidMethodId)
      ));

      // invalid tag
      assert!(matches!(
        DemiaDID::parse(format!("did:{}:", DemiaDID::METHOD)),
        Err(DIDError::InvalidMethodId)
      ));

      // too many segments in method_id
      assert!(matches!(
        DemiaDID::parse(format!("did:{}:{}:test:foo:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)),
        Err(DIDError::InvalidMethodId)
      ));
    };

    execute_assertions(DemiaDID::PLACEHOLDER_TAG);
    execute_assertions(VALID_ALIAS_ID_STR);
  }

  // ===========================================================================================================================
  // Test constructors with randomly generated input
  // ===========================================================================================================================

  #[cfg(feature = "iota-client")]
  fn arbitrary_alias_id() -> impl Strategy<Value = iota_sdk::types::block::output::AliasId> {
    (
      proptest::prelude::any::<[u8; 32]>(),
      iota_sdk::types::block::output::OUTPUT_INDEX_RANGE,
    )
      .prop_map(|(bytes, idx)| {
        let transaction_id = iota_sdk::types::block::payload::transaction::TransactionId::new(bytes);
        let output_id = iota_sdk::types::block::output::OutputId::new(transaction_id, idx).unwrap();
        iota_sdk::types::block::output::AliasId::from(&output_id)
      })
  }

  #[cfg(feature = "iota-client")]
  proptest! {
    #[test]
    fn property_based_valid_parse(alias_id in arbitrary_alias_id()) {
      let did: String = format!("did:{}:{}",DemiaDID::METHOD, alias_id);
      assert!(DemiaDID::parse(did).is_ok());
    }
  }

  #[cfg(feature = "iota-client")]
  proptest! {
    #[test]
    fn property_based_new(bytes in proptest::prelude::any::<[u8;32]>()) {
      for network_name in VALID_NETWORK_NAMES.iter().map(|name| NetworkName::try_from(*name).unwrap()) {
        // check that this does not panic
        DemiaDID::new(&bytes, &CountryCode::USA, &network_name);
      }
    }
  }

  #[cfg(feature = "iota-client")]
  proptest! {
    #[test]
    fn property_based_alias_id_string_representation_roundtrip(alias_id in arbitrary_alias_id()) {
      for network_name in VALID_NETWORK_NAMES.iter().map(|name| NetworkName::try_from(*name).unwrap()) {
        assert_eq!(
          iota_sdk::types::block::output::AliasId::from_str(DemiaDID::new(&alias_id, &CountryCode::USA, &network_name).tag()).unwrap(),
          alias_id
        );
      }
    }
  }

  fn arbitrary_alias_id_string_replica() -> impl Strategy<Value = String> {
    proptest::string::string_regex(&format!("0x([a-f]|[0-9]){{{}}}", (LEN_VALID_ALIAS_STR - 2)))
      .expect("regex should be ok")
  }

  proptest! {
    #[test]
    fn valid_alias_id_string_replicas(tag in arbitrary_alias_id_string_replica()) {
      let did : String = format!("did:{}:{}", DemiaDID::METHOD, tag);
      assert!(
        DemiaDID::parse(did).is_ok()
      );
    }
  }

  fn arbitrary_invalid_tag() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[[:^alpha:]|[a-z]|[1-9]]*")
      .expect("regex should be ok")
      .prop_map(|arb_string| {
        if arb_string
          .chars()
          .all(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase())
          && arb_string.len() == LEN_VALID_ALIAS_STR
          && arb_string.starts_with("0x")
        {
          // this means we are in the rare case of generating a valid string hence we replace the last 0 with the non
          // ascii character é
          let mut counter = 0;
          arb_string
            .chars()
            .rev()
            .map(|value| {
              if value == '0' && counter == 0 {
                counter += 1;
                'é'
              } else {
                value
              }
            })
            .collect::<String>()
        } else {
          arb_string
        }
      })
  }

  proptest! {
    #[test]
    fn invalid_tag_property_based_parse(tag in arbitrary_invalid_tag()) {
      let did: String = format!("did:{}:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), tag);
      assert!(
        DemiaDID::parse(did).is_err()
      );
    }
  }

  fn arbitrary_delimiter_mixed_in_prefix_hex() -> impl Strategy<Value = String> {
    proptest::string::string_regex("0x([a-f]|[:]|[0-9])*").expect("regex should be ok")
  }

  proptest! {
    #[test]
    fn invalid_hex_mixed_with_delimiter(tag in arbitrary_delimiter_mixed_in_prefix_hex()) {
      let did: String = format!("did:{}:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), tag);
      assert!(DemiaDID::parse(did).is_err());
    }
  }

  // ===========================================================================================================================
  // Test getters
  // ===========================================================================================================================
  #[test]
  fn test_network() {
    let execute_assertions = |valid_alias_id: &str| {
      let did: DemiaDID = format!("did:{}:{}", DemiaDID::METHOD, valid_alias_id).parse().unwrap();
      assert_eq!(did.network_str(), DemiaDID::DEFAULT_NETWORK);

      let did: DemiaDID = format!("did:{}:{}:dev:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.network_str(), "dev");

      let did: DemiaDID = format!("did:{}:{}:test:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.network_str(), "test");

      let did: DemiaDID = format!("did:{}:{}:custom:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.network_str(), "custom");
    };

    execute_assertions(DemiaDID::PLACEHOLDER_TAG);
    execute_assertions(VALID_ALIAS_ID_STR);
  }

  #[test]
  fn test_country() {
    let execute_assertions = |valid_alias_id: &str| {
      let did: DemiaDID = format!("did:{}:{}", DemiaDID::METHOD, valid_alias_id).parse().unwrap();
      assert_eq!(did.country_str(), DemiaDID::DEFAULT_COUNTRY);

      // Properly lowercase country
      let did: DemiaDID = format!("did:{}:{}:dev:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.country_str(), "usa");

      // Upper case transformed to lower case
      let did: DemiaDID = format!("did:{}:{}:test:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3(), valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.country_str(), "usa");

      let did: DemiaDID = format!("did:{}:{}:custom:{}", DemiaDID::METHOD, isocountry::alpha3::ISO_A3_USA, valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.country_str(), "usa");
    };

    execute_assertions(DemiaDID::PLACEHOLDER_TAG);
    execute_assertions(VALID_ALIAS_ID_STR);
  }

  #[test]
  fn test_tag() {
    let execute_assertions = |valid_alias_id: &str| {
      let did: DemiaDID = format!("did:{}:{}", DemiaDID::METHOD, valid_alias_id).parse().unwrap();
      assert_eq!(did.tag(), valid_alias_id);

      let did: DemiaDID = format!(
        "did:{}:{}:{}:{}", 
        DemiaDID::METHOD,
        DemiaDID::DEFAULT_COUNTRY,
        DemiaDID::DEFAULT_NETWORK,
        valid_alias_id
      )
      .parse()
      .unwrap();
      assert_eq!(did.tag(), valid_alias_id);

      let did: DemiaDID = format!("did:{}:{}:dev:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.tag(), valid_alias_id);

      let did: DemiaDID = format!("did:{}:{}:custom:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)
        .parse()
        .unwrap();
      assert_eq!(did.tag(), valid_alias_id);
    };
    execute_assertions(DemiaDID::PLACEHOLDER_TAG);
    execute_assertions(VALID_ALIAS_ID_STR);
  }

  // ===========================================================================================================================
  // Test DIDUrl
  // ===========================================================================================================================

  #[test]
  fn test_parse_did_url_valid() {
    let execute_assertions = |valid_alias_id: &str| {
      assert!(DIDUrl::parse(format!("did:{}:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!("did:{}:{}#fragment", DemiaDID::METHOD, valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:{}?somequery=somevalue",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:{}?somequery=somevalue#fragment",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());

      assert!(DIDUrl::parse(format!("did:{}:main:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!("did:{}:main:{}#fragment", DemiaDID::METHOD, valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:main:{}?somequery=somevalue",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:main:{}?somequery=somevalue#fragment",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());

      assert!(DIDUrl::parse(format!("did:{}:dev:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!("did:{}:dev:{}#fragment", DemiaDID::METHOD, valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:dev:{}?somequery=somevalue",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:dev:{}?somequery=somevalue#fragment",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());

      assert!(DIDUrl::parse(format!("did:{}:custom:{}:{}", DemiaDID::METHOD, &CountryCode::USA.alpha3().to_lowercase(), valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!("did:{}:custom:{}#fragment", DemiaDID::METHOD, valid_alias_id)).is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:custom:{}?somequery=somevalue",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());
      assert!(DIDUrl::parse(format!(
        "did:{}:custom:{}?somequery=somevalue#fragment",
        DemiaDID::METHOD,
        valid_alias_id
      ))
      .is_ok());
    };
    execute_assertions(DemiaDID::PLACEHOLDER_TAG);
    execute_assertions(VALID_ALIAS_ID_STR);
  }

  #[test]
  fn valid_url_setters() {
    let execute_assertions = |valid_alias_id: &str| {
      let mut did_url: DIDUrl = DemiaDID::parse(format!("did:{}:{}", DemiaDID::METHOD, valid_alias_id))
        .unwrap()
        .into_url();

      did_url.set_path(Some("/foo")).unwrap();
      did_url.set_query(Some("diff=true")).unwrap();
      did_url.set_fragment(Some("foo")).unwrap();

      assert_eq!(did_url.path(), Some("/foo"));
      assert_eq!(did_url.query(), Some("diff=true"));
      assert_eq!(did_url.fragment(), Some("foo"));
    };
    execute_assertions(DemiaDID::PLACEHOLDER_TAG);
    execute_assertions(VALID_ALIAS_ID_STR);
  }
}
