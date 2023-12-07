
pub mod convert {
    use anyhow::Context;
    use base64::Engine;

    pub fn base64_encode(input: impl AsRef<[u8]>) -> String {
        base64::engine::general_purpose::STANDARD.encode(input.as_ref())
    }

    pub fn base64_decode(input: impl AsRef<[u8]>) -> anyhow::Result<Vec<u8>> {
        base64::engine::general_purpose::STANDARD.decode(input.as_ref())
            .context("Failed to decode base64")
    }

    pub fn base64_decode_string(input: impl AsRef<str>) -> anyhow::Result<String> {
        let bytes = base64_decode(input.as_ref())?;
        String::from_utf8(bytes).context("Failed to decode utf8")
    }

    pub fn json_encode<T: serde::Serialize>(input: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(input)
    }

    pub fn json_decode<T: serde::de::DeserializeOwned>(input: impl AsRef<str>) -> Result<T, serde_json::Error> {
        serde_json::from_str(input.as_ref())
    }
}

pub mod rand {
    use rand::rngs::OsRng;
    use rand::Rng;

    pub fn random_string(length: usize) -> String {
        use rand::distributions::Alphanumeric;

        let mut rng = &mut OsRng;
        rng.sample_iter(Alphanumeric)
            .take(length)
            .map(char::from)
            .collect::<String>()
    }

    pub fn random_string_with_prefix(length: usize, prefix: impl AsRef<str>) -> String {
        let prefix = prefix.as_ref();
        let prefix_length = prefix.len();
        let length = length - prefix_length;
        let random_string = random_string(length);
        format!("{}{}", prefix, random_string)
    }
}