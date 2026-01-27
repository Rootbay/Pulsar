use serde::{Deserialize, Serialize, Serializer, Deserializer};
use zeroize::{Zeroizing};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretString(Zeroizing<String>);

impl std::hash::Hash for SecretString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

impl SecretString {
    pub fn new(s: String) -> Self {
        Self(Zeroizing::new(s))
    }

    pub fn from_zeroized(z: Zeroizing<String>) -> Self {
        Self(z)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SecretString {
    fn default() -> Self {
        Self(Zeroizing::new(String::new()))
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SecretString(Zeroizing::new(s)))
    }
}

impl std::ops::Deref for SecretString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for SecretString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
