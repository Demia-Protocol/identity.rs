// Resolve a legacy (country/network-less) Demia DID and prove it decodes correctly.
//
// Usage:
//   # offline checks only (no node needed):
//   cargo run -p identity_demia_core --example resolve_legacy -- <did> --offline
//
//   # offline checks + live resolution against a node:
//   NODE_URL=http://localhost:14265 \
//   cargo run -p identity_demia_core --example resolve_legacy -- <did>
//
// <did> defaults to the sample legacy DID if omitted.

use identity_demia_core::DemiaDID;
use identity_demia_core::DemiaDocument;
use identity_demia_core::IotaIdentityClientExt;
use identity_did::DID;
use iota_sdk::client::Client;

const DEFAULT_DID: &str = "did:demia:0xbdb06b7c0ee20e293049d76f81fb74fe1caad02faec42e9cfe2641a9086d9a0a";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let offline = args.iter().any(|a| a == "--offline");
  let did_str = args
    .iter()
    .find(|a| !a.starts_with("--"))
    .cloned()
    .unwrap_or_else(|| DEFAULT_DID.to_string());

  // ---- Level 1: offline parse / default checks (no node) ----
  println!("Input DID:        {did_str}");
  let did: DemiaDID = DemiaDID::parse(&did_str)?; // fails here if it doesn't decode at all
  println!("Parsed OK.");
  println!("  canonical str:  {}", did.as_str());
  println!("  method:         {}", did.method());
  println!("  country_str():  {}   (defaulted = {})", did.country_str(), did.country_str() == DemiaDID::DEFAULT_COUNTRY);
  println!("  network_str():  {}   (defaulted = {})", did.network_str(), did.network_str() == DemiaDID::DEFAULT_NETWORK);
  println!("  tag():          {}", did.tag());

  // The canonical string must be byte-identical to what you passed in (no silent rewrite to long form).
  assert_eq!(did.as_str(), did_str.to_lowercase(), "DID string was rewritten during parse!");
  // Validity must pass even though country/network are absent.
  assert!(DemiaDID::is_valid(did.as_ref()), "DID failed validity check");
  println!("Offline checks passed: short DID decodes, defaults applied, no rewrite.\n");

  if offline {
    println!("--offline set; skipping live resolution.");
    return Ok(());
  }

  // ---- Level 2: live resolution against the node ----
  let node_url = std::env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:14265".to_string());
  println!("Resolving against node: {node_url}");
  let client: Client = Client::builder().with_primary_node(&node_url, None)?.finish().await?;

  let document: DemiaDocument = match client.resolve_did(&did).await {
    Ok(doc) => doc,
    Err(e) => {
      eprintln!("resolve_did failed: {e}");
      eprintln!("Retrying WITHOUT the network-name guard (direct Alias Output fetch)...\n");
      // Bypass validate_network: fetch the Alias Output by tag and unpack against the
      // original (short) DID. This proves whether the data is actually on-chain,
      // independent of the dmia/smr network-name check.
      use identity_demia_core::block::output::AliasId;
      use identity_demia_core::IotaIdentityClient;
      let alias_id = AliasId::from(&did);
      let (_, alias_output) = client.get_alias_output(alias_id).await?;
      DemiaDocument::unpack_from_output(&did, &alias_output, true)?
    }
  };
  println!("Resolved document id: {}", document.id());
  // The resolved document id should keep the short form you asked for.
  assert_eq!(document.id().as_str(), did.as_str(), "resolved document id changed form!");
  println!("Resolution OK, document id preserved its short form.\n");
  println!("{document:#}");

  Ok(())
}
