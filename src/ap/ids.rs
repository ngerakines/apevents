use rand::{thread_rng, Rng};
use thiserror::Error;
use url::Url;

use crate::error::ApEventsError;

#[derive(Error, Debug)]
pub enum ObjectIdError {
    #[error("invalid id: {0}")]
    InvalidObjectID(u8),

    #[error("cannot parse object id")]
    CannotParse,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KindType {
    Follow,
    Accept,
}

impl TryFrom<u8> for KindType {
    type Error = ObjectIdError;

    fn try_from(val: u8) -> Result<KindType, ObjectIdError> {
        match val {
            1 => Ok(KindType::Follow),
            2 => Ok(KindType::Accept),
            _ => Err(ObjectIdError::InvalidObjectID(val)),
        }
    }
}

impl TryFrom<Vec<u8>> for KindType {
    type Error = ObjectIdError;

    fn try_from(val: Vec<u8>) -> Result<KindType, ObjectIdError> {
        match val {
            _ if 1 == val[0] => Ok(KindType::Follow),
            _ if 2 == val[0] => Ok(KindType::Accept),
            _ => Err(ObjectIdError::CannotParse),
        }
    }
}

impl KindType {
    pub fn object_prefix(&self) -> String {
        "objects".to_string()
    }

    pub fn as_bytes(&self) -> Result<[u8; 1], ObjectIdError> {
        match self {
            KindType::Follow => Ok(1u8.to_be_bytes()),
            KindType::Accept => Ok(2u8.to_be_bytes()),
        }
    }
}

pub fn generate_object_id(external_base: &str, kind_type: KindType) -> Result<Url, ApEventsError> {
    let bytes = kind_type.as_bytes()?;

    let mut arr: [u8; 20] = [0; 20];
    thread_rng().fill(&mut arr[..]);
    arr[0] = bytes[0];
    let generated_id = base64::encode(arr);

    Url::parse(&format!(
        "{}/{}/{}",
        external_base,
        kind_type.object_prefix(),
        generated_id
    ))
    .map_err(ApEventsError::conv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_kind_from_u8() {
        assert_eq!(
            <u8 as TryInto<KindType>>::try_into(1u8).expect("1 is follow"),
            KindType::Follow
        );
        assert_eq!(
            <u8 as TryInto<KindType>>::try_into(2u8).expect("2 is accept"),
            KindType::Accept
        );
        assert!(<u8 as TryInto<KindType>>::try_into(3u8).is_err());
    }

    #[test]
    fn kind_from_vector() {
        assert_eq!(
            <Vec<u8> as TryInto<KindType>>::try_into(
                base64::decode("AQGFsrFc/j9nBxsU1S/8XfzVoJQ=").expect("object id must decode")
            )
            .expect("id is follow"),
            KindType::Follow
        );
        assert_eq!(
            <Vec<u8> as TryInto<KindType>>::try_into(
                base64::decode("ArDFU9LX4YbQc8FZ+od5OSBvo78=").expect("object id must decode")
            )
            .expect("id is accept"),
            KindType::Accept
        );
    }
}
