import { DIDDoc, DIDResolver, Message } from "didcomm";
import {
  ALICE_DID,
  ALICE_DID_DOC,
  ALICE_SECRETS,
  BOB_DID,
  BOB_DID_DOC,
  BOB_SECRETS,
  CHARLIE_DID,
  CHARLIE_DID_DOC,
  CHARLIE_ROTATED_TO_ALICE_SECRETS,
  ExampleDIDResolver,
  ExampleSecretsResolver,
  IMESSAGE_FROM_PRIOR,
  MESSAGE_FROM_PRIOR,
  IMESSAGE_MINIMAL,
  IMESSAGE_SIMPLE,
  PLAINTEXT_FROM_PRIOR,
  PLAINTEXT_MSG_MINIMAL,
  PLAINTEXT_MSG_SIMPLE,
} from "../test-vectors";

test.each([
  {
    case: "Minimal",
    msg: PLAINTEXT_MSG_MINIMAL,
    options: {},
    expMsg: IMESSAGE_MINIMAL,
    expMetadata: {
      anonymous_sender: false,
      authenticated: false,
      enc_alg_anon: null,
      enc_alg_auth: null,
      encrypted: false,
      encrypted_from_kid: null,
      encrypted_to_kids: null,
      from_prior: null,
      from_prior_issuer_kid: null,
      non_repudiation: false,
      re_wrapped_in_forward: false,
      sign_alg: null,
      sign_from: null,
      signed_message: null,
    },
  },
  {
    case: "Simple",
    msg: PLAINTEXT_MSG_SIMPLE,
    options: {},
    expMsg: IMESSAGE_SIMPLE,
    expMetadata: {
      anonymous_sender: false,
      authenticated: false,
      enc_alg_anon: null,
      enc_alg_auth: null,
      encrypted: false,
      encrypted_from_kid: null,
      encrypted_to_kids: null,
      from_prior: null,
      from_prior_issuer_kid: null,
      non_repudiation: false,
      re_wrapped_in_forward: false,
      sign_alg: null,
      sign_from: null,
      signed_message: null,
    },
  },
  {
    case: "FromPrior",
    msg: PLAINTEXT_FROM_PRIOR,
    options: {},
    expMsg: IMESSAGE_FROM_PRIOR,
    expMetadata: {
      anonymous_sender: false,
      authenticated: false,
      enc_alg_anon: null,
      enc_alg_auth: null,
      encrypted: false,
      encrypted_from_kid: null,
      encrypted_to_kids: null,
      from_prior: {
        aud: "123",
        exp: 1234,
        iat: 123456,
        iss: "did:example:charlie",
        jti: "dfg",
        nbf: 12345,
        sub: "did:example:alice",
      },
      from_prior_issuer_kid: "did:example:charlie#key-1",
      non_repudiation: false,
      re_wrapped_in_forward: false,
      sign_alg: null,
      sign_from: null,
      signed_message: null,
    },
  },
])(
  "Message.unpack works for $case",
  async ({ msg, options, expMsg, expMetadata }) => {
    const didResolver = new ExampleDIDResolver([
      ALICE_DID_DOC,
      BOB_DID_DOC,
      CHARLIE_DID_DOC,
    ]);

    const secretsResolver = new ExampleSecretsResolver(BOB_SECRETS);

    const [unpacked, metadata] = await Message.unpack(
      msg,
      didResolver,
      secretsResolver,
      options
    );

    expect(unpacked.as_value()).toStrictEqual(expMsg);
    expect(metadata).toStrictEqual(expMetadata);
  }
);

test("Message.unpack returns the plaintext as it was written", async () => {
  // Spacing, a duplicate member name and a long decimal do not survive parsing.
  const written = `{ "id": "1", "typ": "application/didcomm-plain+json",
    "type": "http://example.com/protocols/lets_do_lunch/1.0/proposal",
    "body": {"a": 1, "a": 2, "n": 333333333.33333329} }`;

  const [unpacked, , plaintext] = await Message.unpack(
    written,
    new ExampleDIDResolver([ALICE_DID_DOC, BOB_DID_DOC]),
    new ExampleSecretsResolver(BOB_SECRETS),
    {}
  );

  expect(plaintext).toBe(written);
  expect(unpacked.as_value().body.a).toBe(2);
});

test("Message.unpack reads a JSON attachment's numbers as JSON.parse does", async () => {
  const written = `{"id":"1","typ":"application/didcomm-plain+json",
    "type":"http://example.com/protocols/lets_do_lunch/1.0/proposal","body":{},
    "attachments":[{"data":{"json":{"extra":1.797693134862315708e308,"n":333333333.33333329}}}]}`;

  const [unpacked, , plaintext] = await Message.unpack(
    written,
    new ExampleDIDResolver([ALICE_DID_DOC, BOB_DID_DOC]),
    new ExampleSecretsResolver(BOB_SECRETS),
    {}
  );

  expect(plaintext).toBe(written);
  expect(unpacked.as_value().attachments).toStrictEqual(
    JSON.parse(written).attachments
  );
});

test("Message.unpack returns the plaintext from inside an encrypted envelope", async () => {
  const didResolver = new ExampleDIDResolver([ALICE_DID_DOC, BOB_DID_DOC]);
  const message = new Message(IMESSAGE_SIMPLE);
  const [packed] = await message.pack_encrypted(
    BOB_DID,
    ALICE_DID,
    null,
    didResolver,
    new ExampleSecretsResolver(ALICE_SECRETS),
    { forward: false }
  );

  const [unpacked, , plaintext] = await Message.unpack(
    packed,
    didResolver,
    new ExampleSecretsResolver(BOB_SECRETS),
    {}
  );

  expect(plaintext).toBe(await message.pack_plaintext(didResolver));
  expect(unpacked.as_value()).toStrictEqual(IMESSAGE_SIMPLE);
});

class RecordingDIDResolver implements DIDResolver {
  resolved: string[] = [];
  inner: ExampleDIDResolver;

  constructor(knownDids: DIDDoc[]) {
    this.inner = new ExampleDIDResolver(knownDids);
  }

  async resolve(did: string): Promise<DIDDoc | null> {
    this.resolved.push(did);
    return this.inner.resolve(did);
  }
}

test("Message.unpack keeps from_prior unverified if verify_from_prior is false", async () => {
  const [packed] = await MESSAGE_FROM_PRIOR.pack_encrypted(
    BOB_DID,
    ALICE_DID,
    null,
    new ExampleDIDResolver([ALICE_DID_DOC, BOB_DID_DOC, CHARLIE_DID_DOC]),
    new ExampleSecretsResolver(CHARLIE_ROTATED_TO_ALICE_SECRETS),
    { forward: false }
  );

  const didResolver = new RecordingDIDResolver([ALICE_DID_DOC, BOB_DID_DOC]);
  const secretsResolver = new ExampleSecretsResolver(BOB_SECRETS);

  const [unpacked, metadata] = await Message.unpack(
    packed,
    didResolver,
    secretsResolver,
    { verify_from_prior: false }
  );

  expect(unpacked.as_value()).toStrictEqual(IMESSAGE_FROM_PRIOR);
  expect(metadata.authenticated).toBe(true);
  expect(metadata.from_prior).toBeNull();
  expect(metadata.from_prior_issuer_kid).toBeNull();
  expect(didResolver.resolved).not.toContain(CHARLIE_DID);

  const res = Message.unpack(packed, didResolver, secretsResolver, {});
  await expect(res).rejects.toThrowError("from_prior issuer DIDDoc not found");
});
