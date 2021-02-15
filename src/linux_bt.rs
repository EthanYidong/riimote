use bluez_async::*;
use futures::prelude::*;
use tracing::*;

use std::time::Duration;

const WIIMOTE_NAME: &'static str = "Nintendo RVL-CNT-01";

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