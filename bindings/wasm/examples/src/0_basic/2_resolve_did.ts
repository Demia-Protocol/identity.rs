// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Client, MnemonicSecretManager } from "@iota/client-wasm/node";
import { Bip39 } from "@iota/crypto.js";
import { IotaDocument, IotaIdentityClient, JwkMemStore, KeyIdMemStore, Storage } from "@iota/identity-wasm/node";
import type { IAliasOutput } from "@iota/iota.js";
import { API_ENDPOINT, createDid } from "../util";

/** Demonstrates how to resolve an existing DID in an Alias Output. */
export async function resolveIdentity() {
    const client = new Client({
        primaryNode: API_ENDPOINT,
        localPow: true,
    });
    const didClient = new IotaIdentityClient(client);

    // Generate a random mnemonic for our wallet.
    const secretManager: MnemonicSecretManager = {
        mnemonic: Bip39.randomMnemonic(),
    };

    // Creates a new wallet and identity (see "0_create_did" example).
    const storage: Storage = new Storage(new JwkMemStore(), new KeyIdMemStore());
    let { document } = await createDid(
        client,
        secretManager,
        storage,
    );
    const did = document.id();

    // Resolve the associated Alias Output and extract the DID document from it.
    const resolved: IotaDocument = await didClient.resolveDid(did);
    console.log("Resolved DID document:", JSON.stringify(resolved, null, 2));

    // We can also resolve the Alias Output directly.
    const aliasOutput: IAliasOutput = await didClient.resolveDidOutput(did);
    console.log("The Alias Output holds " + aliasOutput.amount + " tokens");
}
