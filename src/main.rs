use bluez::client::BlueZClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = BlueZClient::new().unwrap();

    let version = client.get_mgmt_version().await?;
    println!(
        "management version: {}.{}",
        version.version, version.revision
    );

    let controllers = client.get_controller_list().await?;

    for controller in controllers {
        let info = client.get_controller_info(controller).await?;
        println!("{:?}", info)
    }
    Ok(())
}
