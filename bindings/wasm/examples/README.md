![banner](./../../../.meta/identity_banner.png)

## IOTA Identity Examples

This folder provides code examples for you to learn how the IOTA Identity WASM bindings can be used in JavaScript.

You can run each example using

```
npm run example:node -- <example_name>
```

For Instance, to run the example `create_did`, use

```
npm run example:node -- create_did
```

The following examples are currently available:

| #    | Name                                                   | Information                                                                                                                |
| :--: | :----------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------- |
| 1    | [create_did](create_did.js)                  | Generates and publishes a DID Document, the fundamental building block for decentralized identity.                        |
| 2    | [manipulate_did](manipulate_did.js)          | Add verification methods and service endpoints to a DID Document and update an already existing DID Document.    |
| 3    | [resolution](resolution.js) | Resolves an existing DID to return the latest DID Document. |
| 4    | [create_vc](create_vc.js)      | Generates and publishes subject and issuer DID Documents, then creates a Verifiable Credential (VC) specifying claims about the subject, and verifies it. |
| 5    | [revocation](revocation.js)  | Remove a verification method from the Issuers DID Document, making the Verifiable Credential it signed unable to verify, effectively revoking the VC. |
| 6    | [create_vp](create_vp.js)                            | Create a Verifiable Presentation, the data model for sharing VCs, out of a Verifiable Credential and verifies it. |
| 7    | [merkle_key](merkle_key.js)                            | Adds a MerkleKeyCollection verification method to an Issuers DID Document and signs a Verifiable Credential with the key on index 0. Afterwards the key on index 0 is deactivated, making the Verifiable Credential fail its verification. |