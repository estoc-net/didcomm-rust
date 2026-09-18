use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Attachment {
    /// A JSON object that gives access to the actual content of the attachment.
    /// Can be based on base64, json or external links.
    pub data: AttachmentData,

    /// Identifies attached content within the scope of a given message.
    ///  Recommended on appended attachment descriptors. Possible but generally unused
    ///  on embedded attachment descriptors. Never required if no references to the attachment
    ///  exist; if omitted, then there is no way to refer to the attachment later in the thread,
    ///  in error messages, and so forth. Because id is used to compose URIs, it is recommended
    ///  that this name be brief and avoid spaces and other characters that require URI escaping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// A human-readable description of the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A hint about the name that might be used if this attachment is persisted as a file.
    /// It is not required, and need not be unique. If this field is present and mime-type is not,
    /// the extension on the filename may be used to infer a MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// Describes the MIME type of the attached content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    /// Describes the format of the attachment if the mime_type is not sufficient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// A hint about when the content in this attachment was last modified
    /// in UTC Epoch Seconds (seconds since 1970-01-01T00:00:00Z UTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmod_time: Option<u64>,

    /// Mostly relevant when content is included by reference instead of by value.
    /// Lets the receiver guess how expensive it will be, in time, bandwidth, and storage,
    /// to fully fetch the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_count: Option<u64>,
}

impl Attachment {
    pub fn base64(base64: String) -> AttachmentBuilder {
        AttachmentBuilder::new(AttachmentData::Base64 {
            value: Base64AttachmentData {
                base64,
                hash: None,
                jws: None,
            },
        })
    }

    pub fn json(json: Value) -> AttachmentBuilder {
        AttachmentBuilder::new(AttachmentData::Json {
            value: JsonAttachmentData {
                json,
                hash: None,
                jws: None,
            },
        })
    }

    pub fn links(links: Vec<String>, hash: String) -> AttachmentBuilder {
        AttachmentBuilder::new(AttachmentData::Links {
            value: LinksAttachmentData {
                links,
                hash,
                jws: None,
            },
        })
    }
}

pub struct AttachmentBuilder {
    data: AttachmentData,
    id: Option<String>,
    description: Option<String>,
    filename: Option<String>,
    media_type: Option<String>,
    format: Option<String>,
    lastmod_time: Option<u64>,
    byte_count: Option<u64>,
}

impl AttachmentBuilder {
    fn new(data: AttachmentData) -> Self {
        AttachmentBuilder {
            data,
            id: None,
            description: None,
            filename: None,
            media_type: None,
            format: None,
            lastmod_time: None,
            byte_count: None,
        }
    }

    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn media_type(mut self, media_type: String) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn format(mut self, format: String) -> Self {
        self.format = Some(format);
        self
    }

    pub fn lastmod_time(mut self, lastmod_time: u64) -> Self {
        self.lastmod_time = Some(lastmod_time);
        self
    }

    pub fn byte_count(mut self, byte_count: u64) -> Self {
        self.byte_count = Some(byte_count);
        self
    }

    /// Sets the multi-hash of inline content; a links attachment already carries its hash.
    pub fn hash(mut self, hash: String) -> Self {
        match self.data {
            AttachmentData::Base64 { ref mut value } => value.hash = Some(hash),
            AttachmentData::Json { ref mut value } => value.hash = Some(hash),
            AttachmentData::Links { ref mut value } => value.hash = hash,
        }

        self
    }

    pub fn jws(mut self, jws: Value) -> Self {
        match self.data {
            AttachmentData::Base64 { ref mut value } => value.jws = Some(jws),
            AttachmentData::Json { ref mut value } => value.jws = Some(jws),
            AttachmentData::Links { ref mut value } => value.jws = Some(jws),
        }

        self
    }

    pub fn finalize(self) -> Attachment {
        Attachment {
            data: self.data,
            id: self.id,
            description: self.description,
            filename: self.filename,
            media_type: self.media_type,
            format: self.format,
            lastmod_time: self.lastmod_time,
            byte_count: self.byte_count,
        }
    }
}

/// Represents attachment data in Base64, embedded Json or Links form.
///
/// The three forms are told apart by which carrier member is present. An
/// object carrying two of `base64`, `json` and `links` has no single
/// content, so it is rejected instead of silently reduced to one of them.
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum AttachmentData {
    Base64 {
        #[serde(flatten)]
        value: Base64AttachmentData,
    },
    Json {
        #[serde(flatten)]
        value: JsonAttachmentData,
    },
    Links {
        #[serde(flatten)]
        value: LinksAttachmentData,
    },
}

const CARRIERS: [&str; 3] = ["base64", "json", "links"];

impl<'de> Deserialize<'de> for AttachmentData {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let object = serde_json::Map::<String, Value>::deserialize(deserializer)?;
        let present: Vec<&str> = CARRIERS
            .iter()
            .copied()
            .filter(|carrier| object.contains_key(*carrier))
            .collect();

        let carrier = match present.as_slice() {
            [carrier] => *carrier,
            [] => {
                return Err(serde::de::Error::custom(
                    "attachment data carries none of base64, json or links",
                ))
            }
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "attachment data carries more than one of base64, json or links: {}",
                    present.join(", ")
                )))
            }
        };

        let value = Value::Object(object);
        let data = match carrier {
            "base64" => AttachmentData::Base64 {
                value: serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            },
            "json" => AttachmentData::Json {
                value: serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            },
            _ => AttachmentData::Links {
                value: serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            },
        };

        Ok(data)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Base64AttachmentData {
    /// Base64-encoded data, when representing arbitrary content inline.
    pub base64: String,

    /// The hash of the content encoded in multi-hash format. Used as an integrity check for the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// A JSON Web Signature over the content of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JsonAttachmentData {
    /// Directly embedded JSON data.
    pub json: Value,

    /// The hash of the content encoded in multi-hash format. Used as an integrity check for the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// A JSON Web Signature over the content of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LinksAttachmentData {
    /// A list of one or more locations at which the content may be fetched.
    pub links: Vec<String>,

    /// The hash of the content encoded in multi-hash format. Used as an integrity check for the attachment.
    pub hash: String,

    /// A JSON Web Signature over the content of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<Value>,
}

#[cfg(test)]
mod tests {
    use core::panic;
    use serde_json::json;

    use super::*;

    #[test]
    fn attachment_base64_works() {
        let attachment = Attachment::base64("ZXhhbXBsZQ==".to_owned())
            .id("example-1".to_owned())
            .description("example-1-description".to_owned())
            .filename("attachment-1".to_owned())
            .media_type("message/example".to_owned())
            .format("json".to_owned())
            .lastmod_time(10000)
            .byte_count(200)
            .jws(json!({ "protected": "e30", "signature": "c2ln" }))
            .finalize();

        let data = match attachment.data {
            AttachmentData::Base64 { ref value } => value,
            _ => panic!("data isn't base64."),
        };

        assert_eq!(data.base64, "ZXhhbXBsZQ==");
        assert_eq!(
            data.jws,
            Some(json!({ "protected": "e30", "signature": "c2ln" }))
        );
        assert_eq!(attachment.id, Some("example-1".to_owned()));

        assert_eq!(
            attachment.description,
            Some("example-1-description".to_owned())
        );

        assert_eq!(attachment.filename, Some("attachment-1".to_owned()));
        assert_eq!(attachment.media_type, Some("message/example".to_owned()));
        assert_eq!(attachment.format, Some("json".to_owned()));
        assert_eq!(attachment.lastmod_time, Some(10000));
        assert_eq!(attachment.byte_count, Some(200));
    }

    #[test]
    fn attachment_json_works() {
        let attachment = Attachment::json(json!("example"))
            .id("example-1".to_owned())
            .description("example-1-description".to_owned())
            .filename("attachment-1".to_owned())
            .media_type("message/example".to_owned())
            .format("json".to_owned())
            .lastmod_time(10000)
            .byte_count(200)
            .jws(json!({ "protected": "e30", "signature": "c2ln" }))
            .finalize();

        let data = match attachment.data {
            AttachmentData::Json { ref value } => value,
            _ => panic!("data isn't json."),
        };

        assert_eq!(data.json, json!("example"));
        assert_eq!(
            data.jws,
            Some(json!({ "protected": "e30", "signature": "c2ln" }))
        );
        assert_eq!(attachment.id, Some("example-1".to_owned()));

        assert_eq!(
            attachment.description,
            Some("example-1-description".to_owned())
        );

        assert_eq!(attachment.filename, Some("attachment-1".to_owned()));
        assert_eq!(attachment.media_type, Some("message/example".to_owned()));
        assert_eq!(attachment.format, Some("json".to_owned()));
        assert_eq!(attachment.lastmod_time, Some(10000));
        assert_eq!(attachment.byte_count, Some(200));
    }

    #[test]
    fn inline_hash_and_object_jws_survive_a_round_trip() {
        let wire = json!({
            "id": "a1",
            "data": {
                "base64": "aGk",
                "hash": "zQmYmVjaWFs",
                "jws": {
                    "protected": "e30",
                    "signature": "c2ln",
                    "header": { "kid": "did:example:1#key-1" }
                }
            }
        });

        let attachment: Attachment = serde_json::from_value(wire.clone()).expect("deserialize");
        let data = match attachment.data {
            AttachmentData::Base64 { ref value } => value,
            _ => panic!("data isn't base64."),
        };

        assert_eq!(data.hash, Some("zQmYmVjaWFs".to_owned()));
        assert_eq!(data.jws, Some(wire["data"]["jws"].clone()));
        assert_eq!(serde_json::to_value(&attachment).expect("serialize"), wire);

        let wire = json!({ "data": { "json": { "note": null }, "hash": "zQmYmVjaWFs" } });
        let attachment: Attachment = serde_json::from_value(wire.clone()).expect("deserialize");
        assert_eq!(serde_json::to_value(&attachment).expect("serialize"), wire);
    }

    #[test]
    fn attachment_data_with_two_carriers_is_rejected() {
        for data in [
            json!({ "base64": "aGk", "json": { "different": true } }),
            json!({ "base64": "aGk", "json": null }),
            json!({ "base64": "aGk", "links": ["https://example.invalid/other"], "hash": "zQmYmVjaWFs" }),
            json!({ "json": { "note": null }, "links": ["https://example.invalid/other"], "hash": "zQmYmVjaWFs" }),
            json!({ "hash": "zQmYmVjaWFs" }),
        ] {
            let result = serde_json::from_value::<AttachmentData>(data.clone());
            assert!(result.is_err(), "{data} should not deserialize");
        }

        let alone: AttachmentData =
            serde_json::from_value(json!({ "json": null })).expect("a null json payload");
        assert_eq!(
            alone,
            AttachmentData::Json {
                value: JsonAttachmentData {
                    json: Value::Null,
                    hash: None,
                    jws: None,
                }
            }
        );
    }

    #[test]
    fn attachment_links_works() {
        let attachment = Attachment::links(
            vec!["http://example1".to_owned(), "https://example2".to_owned()],
            "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c".to_owned(),
        )
        .id("example-1".to_owned())
        .description("example-1-description".to_owned())
        .filename("attachment-1".to_owned())
        .media_type("message/example".to_owned())
        .format("json".to_owned())
        .lastmod_time(10000)
        .byte_count(200)
        .jws(json!({ "protected": "e30", "signature": "c2ln" }))
        .finalize();

        let data = match attachment.data {
            AttachmentData::Links { ref value } => value,
            _ => panic!("data isn't links."),
        };

        assert_eq!(
            data.links,
            vec!["http://example1".to_owned(), "https://example2".to_owned()]
        );

        assert_eq!(
            data.hash,
            "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c".to_owned()
        );

        assert_eq!(
            data.jws,
            Some(json!({ "protected": "e30", "signature": "c2ln" }))
        );
        assert_eq!(attachment.id, Some("example-1".to_owned()));

        assert_eq!(
            attachment.description,
            Some("example-1-description".to_owned())
        );

        assert_eq!(attachment.filename, Some("attachment-1".to_owned()));
        assert_eq!(attachment.media_type, Some("message/example".to_owned()));
        assert_eq!(attachment.format, Some("json".to_owned()));
        assert_eq!(attachment.lastmod_time, Some(10000));
        assert_eq!(attachment.byte_count, Some(200));
    }
}
