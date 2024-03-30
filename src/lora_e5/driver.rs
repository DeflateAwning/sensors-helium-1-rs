use arrayvec::ArrayVec;
use byteorder::{BigEndian, ByteOrder};
use core::cell::RefCell;
use embedded_hal::serial::{Read, Write};
use nb::block;

use crate::lora_e5::commands::Command;
use crate::lora_e5::responses::{Reply};
use crate::lora_e5::responses;
use crate::lora_e5::utils::{CommandWriter, Error, FromPayload, ToPayload};

const MAX_REPLY_LENGTH: usize = 1024;


/// Represents a LoRa-E5 device connected to a U(S)ART.
#[derive(Debug)]
pub struct LoRaE5<TX, RX> {
    tx: TX,
    rx: RX,
    received: ArrayVec<u8, 1024>,
    cmd_buffer: ArrayVec<u8, 128>,
    inflight_request: RefCell<Option<Command>>,
}

impl<TX, RX> CommandWriter for LoRaE5<TX, RX> {
    fn write_cmd_bytes(&mut self, bytes: &[u8]) {
        self.cmd_buffer.try_extend_from_slice(bytes).unwrap();
    }
}

impl<TX, RX> LoRaE5<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    /// Creates an instance of the LoRaE5. `tx` and `rx` are the transmit and receive halves of a
    /// USART.
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            tx: tx,
            rx: rx,
            received: ArrayVec::<u8, 1024>::new(),
            cmd_buffer: ArrayVec::<u8, 128>::new(),
            inflight_request: RefCell::from(None),
        }
    }

    /// Sends a command `cmd` to the LoRaE5 and then blocks waiting for the reply.
    /// The return value is either a response from the LoRaE5 or an error. Uses blocking USART
    /// API.
    ///
    /// # Errors
    ///
    /// ## `Error::WriteError(err)`
    /// Returned if the command could not be written to the serial port.
    /// Wraps the underlying error.
    ///
    /// ## `Error::ReadError(err)`
    /// Returned if the reply could not be read from the serial port.
    /// Wraps the underlying error.
    ///
    /// ## `Error::RecvPacketTooShort`
    /// Returned if the reply was only partially received.
    ///
    /// ## `Error::RecvWrongReplyType`
    /// Returned if the response packet was not a reply.
    pub fn send_command(&mut self, cmd: Command) -> Result<Reply, Error<TX::Error, RX::Error>> {
        self.cmd_buffer.clear();
        self.received.clear();
        self.prepare_cmd(cmd);

        let cmd_bytes = &self.cmd_buffer[..];
        for byte in cmd_bytes {
            match block!(self.tx.write(*byte)) {
                Err(e) => return Err(Error::WriteError(e)),
                Ok(..) => {}
            }
        }

        match block!(self.tx.flush()) {
            Err(e) => return Err(Error::WriteError(e)),
            Ok(..) => {}
        }

        return self.read_reply().and_then(|_| self.parse_reply());
    }

    fn prepare_cmd(&mut self, cmd: Command) {
        cmd.to_payload(self);
        *self.inflight_request.borrow_mut() = Some(cmd);
    }

    fn read_reply(&mut self) -> Result<u16, Error<TX::Error, RX::Error>> {
        // let mut length_bytes: u16 = 0;
        for _ in 0..MAX_REPLY_LENGTH {
            match block!(self.rx.read()) {
                Ok(b'\r') => continue, // ignore CR
                Ok(b'\n') => break,
                Ok(word) => {
                    self.received.push(word);
                },
                Err(error) => return Err(Error::RecvReadError(error)),
            }
        }

        // TODO: figure out how this ends

        return Ok(self.received.len() as u16);
    }

    fn parse_reply(&self) -> Result<Reply, Error<TX::Error, RX::Error>> {
        // We have no business reading anything if there's no request in flight
        let inflight = self.inflight_request.borrow();
        if inflight.is_none() {
            return Err(Error::RecvUnsolicitedReply);
        }

        // Check that it starts with "+AT: " (the response format)
        let expected_resp_start = b"+AT: ";
        if self.received.len() < expected_resp_start.len() {
            return Err(Error::RecvPacketTooShort);
        }
        // if !self.received.starts_with(expected_resp_start) { // FIXME: make this check work; maybe log the received bytes when this failure happens
        //     return Err(Error::RecvWrongReplyType);
        // }

        return match *inflight {
            Some(Command::CheckAlive) => Ok(Reply::CheckAlive(responses::CheckAliveResult::from_payload(
                &self.received[..],
            ))),

            Some(Command::AT_ID_READ) => Ok(Reply::AT_ID_READ(responses::AT_ID_READ_Result::from_payload(
                &self.received[..],
            ))),
            
            None => panic!("Should not be reached"),
            _ => unimplemented!(),
        };
    }
}
