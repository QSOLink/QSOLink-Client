use qsolink_client::Qso;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let qsos: Vec<Qso> = reqwest::Client::new()
        .get("http://bartmoss.uid0.online:5001/api/qso")
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", qsos);

    Ok(())
}
