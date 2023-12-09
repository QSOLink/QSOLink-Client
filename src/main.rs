
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let qsos = reqwest::Client::new()
        .get("http://bartmoss.uid0.online:5001/api/qso")
        .send()
        .await?
        .text()
        .await?;

    println!("{:#?}", qsos);

       Ok(()) 
}
