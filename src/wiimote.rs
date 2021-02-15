use hidapi::*;

use std::time::Duration;
use std::thread::sleep;

use crate::Report;

const WRITE_DELAY: Duration = Duration::from_millis(10);

pub struct Wiimote {
    inner: HidDevice,
    rumble: bool,
}

impl Wiimote {
    pub fn new(inner: HidDevice) -> Wiimote {
        Wiimote {
            inner,
            rumble: false,
        }
    }

    pub fn rumble(&mut self, rumble: bool) {
        self.rumble = rumble;
    }

    pub fn write(&self, bytes: &mut [u8]) -> HidResult<usize> {
        if self.rumble {
            bytes[1] = bytes[1] | 1;
        } else {
            bytes[1] = bytes[1] & !1;
        }
        self.inner.write(bytes)
    }

    pub fn write_registers(&self, addr: u32, data: & [u8]) -> HidResult<usize> {
        let mut bytes = [0; 22];
        bytes[0] = Report::WriteMemoryAndRegisters as u8;
        bytes[1] = 0x04;
        bytes[2..5].copy_from_slice(&addr.to_be_bytes()[1..]);
        let data_len = 16.min(data.len());
        bytes[5] = data_len as u8;
        bytes[6..6 + data_len].copy_from_slice(&data[0..data_len]);

        let ret = self.write(&mut bytes);
        sleep(WRITE_DELAY);
        ret
    }

    pub fn read(&self, buf: &mut [u8]) -> HidResult<usize> {
        self.inner.read(buf)
    }

    pub fn setup_speakers(&self) -> HidResult<()> {
        self.write(&mut [Report::SpeakerEnable as u8, 0x04])?;
        self.write(&mut [Report::SpeakerMute as u8, 0x04])?;
        self.write_registers(0x00a20009, &[0x01])?;
        self.write_registers(0x00a20001, &[0x08])?;
        self.write_registers(0x00a20001, &[0x00, 0x40, 0x70, 0x17, 0x10, 0x00, 0x00])?;
        self.write_registers(0x00a20008, &[0x01])?;
        self.write(&mut [Report::SpeakerMute as u8, 0x00])?;
        Ok(())
    }

    pub fn play_audio(&self, audio: &[u8]) -> HidResult<()> {
        for chunk in audio.chunks(20) {
            let mut to_send = [0; 22];
            to_send[0] = Report::SpeakerData as u8;
            to_send[1] = 20 << 3;
            to_send[2..2+chunk.len()].copy_from_slice(chunk);
            self.write(&mut to_send)?;
            sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}
