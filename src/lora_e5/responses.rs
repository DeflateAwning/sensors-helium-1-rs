use crate::lora_e5::utils::FromPayload;
use byteorder::{BigEndian, ByteOrder};
use arrayvec::{ArrayString, ArrayVec};

/// Responses to commands returned by the R502. Names are the same as commands.
#[derive(Debug)]
pub enum Reply {
    CheckAlive(CheckAliveResult),
    
    AT_ID_READ(AT_ID_READ_Result),

    AT_ID_SET_DEV_EUI,
    
    AT_ID_SET_APP_EUI,

    AT_ID_SET_APP_KEY,

}

/// Result struct for the `CheckAlive` call
/// 
#[derive(Debug)]
pub struct CheckAliveResult {
    pub full_response: ArrayVec<u8, 100>,
    pub full_response_str: ArrayString<100>,
}

impl FromPayload for CheckAliveResult {
    fn from_payload(payload: &[u8]) -> Self {
        let mut full_response: ArrayVec<u8, 100> = ArrayVec::new();
        full_response.try_extend_from_slice(payload).unwrap(); // FIXME: I still don't like this

        let mut full_response_str: ArrayString<100> = ArrayString::new();
        full_response_str.push_str(core::str::from_utf8(&payload).unwrap());

        return Self {
            full_response: full_response,
            full_response_str: full_response_str,
        };
    }
}



/// Result struct for the `AT_ID_READ` call
#[derive(Debug)]
pub struct AT_ID_READ_Result {
    pub full_response: ArrayVec<u8, 100>,
    pub full_response_str: ArrayString<100>,
}


impl FromPayload for AT_ID_READ_Result {
    fn from_payload(payload: &[u8]) -> Self {
        let mut full_response: ArrayVec<u8, 100> = ArrayVec::new();
        full_response.try_extend_from_slice(payload).unwrap(); // FIXME: I still don't like this

        let mut full_response_str: ArrayString<100> = ArrayString::new();
        full_response_str.push_str(core::str::from_utf8(&payload).unwrap());

        return Self {
            full_response: full_response,
            full_response_str: full_response_str,
        };
    }
}

