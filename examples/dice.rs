use hidapi::*;

use rand::prelude::*;

use tracing::*;

use std::time::Duration;
use std::thread::sleep;

use riimote::*;

const DATA_REPORT_TYPE: u8 = Report::CoreButtonsAccelerometer as u8;

// Generated with ffmpeg -i in.aiff -acodec pcm_s8 -f s8 -ac 1 -ar 2000 out.pcm
const DICE: &'static [u8] = include_bytes!("media/dice.pcm");
const DOUBLE: &'static [u8] = include_bytes!("media/double.pcm");
const NUMBERS: [&'static [u8]; 11] = [
    include_bytes!("media/2.pcm"),
    include_bytes!("media/3.pcm"),
    include_bytes!("media/4.pcm"),
    include_bytes!("media/5.pcm"),
    include_bytes!("media/6.pcm"),
    include_bytes!("media/7.pcm"),
    include_bytes!("media/8.pcm"),
    include_bytes!("media/9.pcm"),
    include_bytes!("media/10.pcm"),
    include_bytes!("media/11.pcm"),
    include_bytes!("media/12.pcm"),
];

fn update_volume(wiimote: &Wiimote, volume: u8) -> anyhow::Result<()> {
    wiimote.write(&mut [Report::PlayerLeds as u8, 1 << (3 + volume)])?;
    wiimote.write_registers(0x00a20005, &[0x20 * volume])?;
    Ok(())
}

fn wiimote_setup(wiimote: &Wiimote) -> anyhow::Result<()> {
    wiimote.write(&mut [Report::DataReportingMode as u8, 0x00, DATA_REPORT_TYPE])?;
    // Speaker initialization
    wiimote.setup_speakers()?;
    Ok(())
}

fn wiimote_loop() -> anyhow::Result<()> {
    let hid = HidApi::new().unwrap();
    let mut wiimote = Wiimote::new(hid.open(0x057E, 0x0306)?);
    wiimote_setup(&wiimote)?;

    let mut input = [0; 22];
    let mut buttons = ButtonFlags::empty();
    let mut accel = [0; 3];
    let mut rng = thread_rng();
    let mut volume = 1;

    loop {
        let bytes_read = wiimote.read(&mut input)?;
        match input[0] {
            DATA_REPORT_TYPE => {
                let new_buttons = ButtonFlags::from_bytes(&input[1..3])?;
                let new_accel = &input[3..6];

                let just_pressed = new_buttons - buttons;

                if just_pressed.contains(ButtonFlags::MINUS) {
                    volume = 1.max(volume - 1);
                    update_volume(&wiimote, volume)?;
                }
                if just_pressed.contains(ButtonFlags::PLUS) {
                    volume = 4.min(volume + 1);
                    update_volume(&wiimote, volume)?;
                }

                if new_buttons.contains(ButtonFlags::A) {
                    if accel[2] < 200 && new_accel[2] > 200 {
                        debug!("Rolling dice");
                        let dice_0 = rng.gen_range(0..6);
                        let dice_1 = rng.gen_range(0..6);
                        let total = dice_0 + dice_1;
                        wiimote.write(&mut [Report::PlayerLeds as u8, (total + 2) << 4])?;
                        wiimote.play_audio(DICE)?;
                        if dice_0 == dice_1 {
                            wiimote.rumble(true);
                            wiimote.play_audio(&DOUBLE)?;
                            wiimote.rumble(false);
                            wiimote.write(&mut [Report::Rumble as u8, 0])?;
                            sleep(Duration::from_millis(300));
                        }
                        wiimote.play_audio(NUMBERS[total as usize])?;
                    }
                }

                buttons = new_buttons;
                accel.copy_from_slice(new_accel);
            }
            _ => {
                trace!("{:02x?}", &input[..bytes_read]);
            }
        }
        sleep(Duration::from_millis(10));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();
    
    let wiimote_search = WiimoteSearch::new().await?;
    let wiimote = wiimote_search.find_wiimote_device_id().await?;

    loop {
        wiimote_search.connect_to_wiimote(&wiimote).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;

        if tokio::task::spawn_blocking(wiimote_loop).await?.is_ok() {
            break;
        }

        info!("Disconnected from wiimote");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
