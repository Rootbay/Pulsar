use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::secmem::LockedString;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretString(LockedString);

impl std::hash::Hash for SecretString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

impl SecretString {
    #[allow(dead_code)]
    pub fn new(s: String) -> Self {
        let mut s = s;
        let locked = LockedString::new(&s);
        s.zeroize();
        Self(locked)
    }

    /// Upgrades a Zeroizing<String> into a page-locked SecretString
    pub fn from_zeroized(z: Zeroizing<String>) -> Self {
        let locked = LockedString::new(&z);
        Self(locked)
    }

    /// Accesses the underlying locked string slice
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for SecretString {
    fn default() -> Self {
        Self(LockedString::new(""))
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut s_mut = s;
        let locked = LockedString::new(&s_mut);
        s_mut.zeroize();
        Ok(SecretString(locked))
    }
}

impl std::ops::Deref for SecretString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl AsRef<str> for SecretString {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
