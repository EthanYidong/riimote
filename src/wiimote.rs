use bluez_async::*;
use hidapi::*;

use futures::prelude::*;

use tracing::*;

use std::time::Duration;
use std::thread::sleep;

use crate::Report;

const WIIMOTE_NAME: &'static str = "Nintendo RVL-CNT-01";
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

pub struct WiimoteSearch {
    session: BluetoothSession
}

impl WiimoteSearch {
    pub async fn new() -> Result<Self, BluetoothError> {
        let (_handle, session) = BluetoothSession::new().await?;

        Ok(WiimoteSearch {
            session
        })
    }

    pub async fn find_wiimote_device_id(&self) -> Result<DeviceId, BluetoothError> {
        let devices = self.session.get_devices().await?;
        let maybe_wiimote = devices
            .into_iter()
            .find(|d| d.name.as_deref() == Some(WIIMOTE_NAME));
        if let Some(wiimote) = maybe_wiimote {
            return Ok(wiimote.id);
        } else {
            info!("Searching for wiimotes");
            self.session.start_discovery().await?;
            let mut event_stream = self.session.event_stream().await?;
            while let Some(ev) = event_stream.next().await {
                match ev {
                    BluetoothEvent::Device {
                        id,
                        event: DeviceEvent::Discovered,
                    } => {
                        let device_info = self.session.get_device_info(&id).await?;
                        if device_info.name.as_deref() == Some(WIIMOTE_NAME) {
                            info!("Found wiimote");
                            self.session.stop_discovery().await?;
                            return Ok(id);
                        }
                    },
                    _ => {}
                }
            }
        }
        panic!("Session stopped");
    }

    pub async fn connect_to_wiimote(&self, wiimote: &DeviceId) -> Result<(), BluetoothError> {
        let wiimote_info = self.session.get_device_info(wiimote).await?;
        if !wiimote_info.connected {
            info!("Attempting to connect to wiimote");
            while let Err(_) = self.session.connect(wiimote).await {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            info!("Connected to wiimote");
        } else {
            info!("Already connected to wiimote");
        }
        Ok(())
    }
}
