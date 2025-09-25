#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub data: [u64; 13],
}

impl Default for Registers {
    fn default() -> Self {
        Self { data: [0; 13] }
    }
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> u64 {
        self.data[index]
    }

    pub fn set(&mut self, index: usize, value: u64) {
        self.data[index] = value;
    }

    pub fn from_slice(slice: [u64; 13]) -> Self {
        Self { data: slice }
    }

    pub fn from_vec(vec: Vec<u64>) -> Self {
        let mut data = [0u64; 13];
        let len = vec.len().min(13);
        data[..len].copy_from_slice(&vec[..len]);
        Self { data }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Registers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Registers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = <[u64; 13]>::deserialize(deserializer)?;
        Ok(Registers { data })
    }
}
