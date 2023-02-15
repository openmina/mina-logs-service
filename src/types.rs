use std::{collections::BTreeMap, time::SystemTime};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    #[serde(with = "rfc3339")]
    timestamp: SystemTime,
    level: Level,
    source: Source,
    message: String,
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Level {
    Error,
    Warning,
    Info,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    module: String,
    location: String,
}


mod rfc3339 {
    use std::time::SystemTime;
    use serde::{Serializer, Deserializer};


    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        todo!()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error> where D: Deserializer<'de> {
        todo!()
    }

    #[cfg(test)]
    mod tests {
        use std::time::SystemTime;

        use serde::{Serialize, Deserialize};



        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct T {
            #[serde(with = "crate::types::rfc3339")]
            time: SystemTime
        }

        #[test]
        fn serialize() {

        }

    }

}
