import { Message, IMessage } from "didcomm";

test("Message.new works", () => {
  const val: IMessage = {
    id: "example-1",
    typ: "application/didcomm-plain+json",
    type: "example/v1",
    body: "example-body",
    from: "did:example:4",
    to: ["did:example:1", "did:example:2", "did:example:3"],
    thid: "example-thread-1",
    pthid: "example-parent-thread-1",
    "example-header-1": "example-header-1-value",
    "example-header-2": "example-header-2-value",
    created_time: 10000,
    expires_time: 20000,
    attachments: [
      {
        data: {
          base64: "ZXhhbXBsZQ==",
        },
        id: "attachment1",
      },
      {
        data: {
          json: "example",
        },
        id: "attachment2",
      },
      {
        data: {
          json: "example",
        },
        id: "attachment3",
      },
    ],
  };

  const msg = new Message(val);
  expect(msg.as_value()).toStrictEqual(val);
});

test("Message.new keeps an inline attachment hash and an object jws", () => {
  const val: IMessage = {
    id: "example-2",
    typ: "application/didcomm-plain+json",
    type: "example/v1",
    body: {},
    attachments: [
      {
        id: "attachment1",
        data: {
          base64: "ZXhhbXBsZQ==",
          hash: "zQmYmVjaWFs",
          jws: {
            protected: "e30",
            signature: "c2ln",
            header: { kid: "did:example:1#key-1" },
          },
        },
      },
      {
        id: "attachment2",
        data: { json: { note: null }, hash: "zQmYmVjaWFs" },
      },
    ],
  };

  const msg = new Message(val);
  expect(msg.as_value()).toStrictEqual(val);
});

test("Message.new rejects attachment data carrying two content forms", () => {
  const val: IMessage = {
    id: "example-3",
    typ: "application/didcomm-plain+json",
    type: "example/v1",
    body: {},
    attachments: [
      {
        id: "ambiguous",
        data: { base64: "aGk", json: { different: true } } as any,
      },
    ],
  };

  expect(() => new Message(val)).toThrow(/Malformed/);
});
