use crate::lora_e5::utils::{CommandWriter, ToPayload};

/// Commands that one can send to the LoRaE5.
///
/// Source: https://wiki.seeedstudio.com/LoRa-E5_STM32WLE5JC_Module/#12-basic-at-commands
#[derive(Debug)]
pub enum Command {
    CheckAlive,

    AT_ID_READ,

    AT_ID_SET_DEV_EUI {
        dev_eui: [u8; 8],
    },
    
    AT_ID_SET_APP_EUI {
        app_eui: [u8; 8],
    },

    AT_ID_SET_APP_KEY {
        app_key: [u8; 16],
    },

    // TODO: many more
}


impl ToPayload for Command {
    fn to_payload(&self, writer: &mut dyn CommandWriter) {
        match self {
            Self::CheckAlive => {
                writer.write_cmd_bytes("AT".as_bytes());
                writer.write_cmd_bytes("\n".as_bytes()); // TODO: maybe factor elsewhere
            }
            
            Self::AT_ID_READ => {
                writer.write_cmd_bytes("AT+ID".as_bytes());
                writer.write_cmd_bytes("\n".as_bytes()); // TODO: maybe factor elsewhere
            }

            // TODO: implement the rest

            _ => unimplemented!(),
        }
    }
}
