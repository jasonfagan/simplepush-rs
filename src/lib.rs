//! # SimplePush: A client for sending notifications via the simplepush.io API.
//!
//! See <https://simplepush.io> for more details.
extern crate crypto;
extern crate rand;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::{aes, blockmodes, buffer, symmetriccipher};
use rand::Rng;
use serde::Serialize;

// Default encryption salt, you should really provide your own
static DEFAULT_SALT: &str = "A9F361C70BCB6182";

// API endpoint
static API_URL: &str = "https://api.simplepush.io";

/// SimplePush API payload
#[derive(Serialize)]
struct Payload {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iv: Option<String>,
}

/// SimplePush message data
pub struct Message {
    /// Your simplepush.io key
    pub key: String,
    /// Title of the message
    pub title: Option<String>,
    /// Message body
    pub message: String,
    /// The event the message should be associated with
    pub event: Option<String>,
    /// Actions for feedback message, only simple actions are supported at this time
    pub actions: Option<Vec<String>>,
    /// If true, the message will be sent with end-to-end encrypted using the provided password & salt
    pub encrypt: bool,
    /// Password if the message is to be encrypted
    pub password: Option<String>,
    /// If set, this salt will be used for encryption, otherwise the `DEFAULT_SALT` will be used
    pub salt: Option<String>,
}

impl Message {
    /// Create a new notification message
    pub fn new(
        key: &str,
        title: Option<&str>,
        message: &str,
        event: Option<&str>,
        actions: Option<Vec<&str>>,
    ) -> Self {
        Message {
            key: String::from(key),
            title: Self::stringify(title),
            message: String::from(message),
            event: Self::stringify(event),
            actions: Self::stringify_vec(actions),
            encrypt: false,
            password: None,
            salt: None,
        }
    }

    /// Create a new notification message with encryption
    pub fn new_with_encryption(
        key: &str,
        title: Option<&str>,
        message: &str,
        event: Option<&str>,
        actions: Option<Vec<&str>>,
        password: &str,
        salt: Option<&str>,
    ) -> Self {
        Message {
            key: String::from(key),
            title: Self::stringify(title),
            message: String::from(message),
            event: Self::stringify(event),
            actions: Self::stringify_vec(actions),
            encrypt: true,
            password: Some(String::from(password)),
            salt: Self::stringify(salt.or(Some(DEFAULT_SALT))),
        }
    }

    fn stringify(s: Option<&str>) -> Option<String> {
        s.map(String::from)
    }

    fn stringify_vec(s: Option<Vec<&str>>) -> Option<Vec<String>> {
        s.map(|t| t.iter().map(|s| String::from(*s)).collect())
    }
}

/// A client for the simplepush.io API
///
/// See: <https://simplepush.io/api> for more details
///
pub struct SimplePush;

impl SimplePush {
    fn encrypt(
        key: &[u8],
        iv: &[u8],
        buf: Vec<u8>,
    ) -> Result<String, symmetriccipher::SymmetricCipherError> {
        let mut encryptor =
            aes::cbc_encryptor(aes::KeySize::KeySize128, key, iv, blockmodes::PkcsPadding);
        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(&buf);
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;
            final_result.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .copied(),
            );

            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }

        Ok(URL_SAFE.encode(final_result))
    }

    fn process_message(message: &Message) -> Payload {
        let message_iv: Option<String>;
        let encrypted: Option<bool>;
        let msg: String;
        let title: Option<String>;
        let actions: Option<Vec<String>>;

        if message.encrypt {
            let salt = message.salt.to_owned().expect("salt was None");
            let password = message.password.to_owned().expect("password was None");
            let mut hasher = Sha1::new();
            hasher.input_str(format!("{}{}", password, salt).as_str());

            let mut key = [0u8; 40];
            hasher.result(&mut key);

            let mut iv = [0u8; 16];
            let mut rng = rand::rngs::OsRng;
            rng.fill(&mut iv[..]);

            msg = SimplePush::encrypt(&key[0..16], &iv, message.message.to_owned().into_bytes())
                .expect("encryption failed!");

            title = message.title.to_owned().map(|t| {
                SimplePush::encrypt(&key[0..16], &iv, t.into_bytes()).expect("encryption failed")
            });

            actions = message.actions.to_owned().map(|t| {
                t.iter()
                    .map(|s| {
                        SimplePush::encrypt(&key[0..16], &iv, s.clone().into_bytes())
                            .expect("encryption failed")
                    })
                    .collect()
            });

            message_iv = Some(SimplePush::hexify(iv.to_vec()).to_ascii_uppercase());
            encrypted = Some(true);
        } else {
            msg = message.message.to_owned();
            title = message.title.to_owned();
            actions = message.actions.to_owned();
            encrypted = None;
            message_iv = None;
        }

        Payload {
            key: message.key.to_owned(),
            title,
            msg,
            event: message.event.to_owned(),
            actions,
            encrypted: encrypted.map(|v| v.to_string()),
            iv: message_iv,
        }
    }

    fn hexify(bytes: Vec<u8>) -> String {
        let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
        strs.join("")
    }

    fn validate(message: &Message) -> Result<(), String> {
        if message.key.is_empty() {
            return Err(String::from("key is required"));
        }

        if message.title.is_none() && message.message.is_empty() {
            return Err(String::from("a message or title is required"));
        }

        if message.encrypt && message.password.is_none()
            || message.password.as_ref().is_some_and(|p| p.is_empty())
        {
            return Err(String::from("password is required for encryption"));
        }

        Ok(())
    }

    /// Sends a notification message through the simplepush.io API
    ///
    /// Sending a notification
    ///
    /// ```no_run
    ///    use simplepush_rs::{Message, SimplePush};
    ///    let _ = SimplePush::send(Message::new(
    ///         "SIMPLE_PUSH_KEY",
    ///         Some("title"),
    ///         "test message",
    ///         Some("alert"),
    ///         Some(vec!["yes", "no"]),
    ///     ));
    ///```
    ///
    /// Sending a notification with encryption
    ///
    /// ```no_run
    ///    use simplepush_rs::{Message, SimplePush};
    ///    let _ = SimplePush::send(Message::new_with_encryption(
    ///         "SIMPLE_PUSH_KEY",
    ///         Some("title"),
    ///         "test message",
    ///         Some("alert"),
    ///         Some(vec!["yes", "no"]),
    ///         "ENCRYPTION_KEY",
    ///         Some("SALT"),
    ///     ));
    ///```
    ///
    pub fn send(message: Message) -> Result<(), String> {
        SimplePush::validate(&message)?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/send", API_URL))
            .json(&SimplePush::process_message(&message))
            .send();
        match response {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_key() {
        let result = SimplePush::send(Message::new("", Some("title"), "message", None, None));
        assert!(result.is_err_and(|e| e == *"key is required"));
    }

    #[test]
    fn test_empty_message() {
        let result = SimplePush::send(Message::new("key", None, "", None, None));
        assert!(result.is_err_and(|e| e == *"a message or title is required"));
    }

    #[test]
    fn test_empty_password_with_encryption() {
        let result = SimplePush::send(Message::new_with_encryption(
            "key", None, "message", None, None, "", None,
        ));
        assert!(result.is_err_and(|e| e == *"password is required for encryption"));
    }
}
