use reqwest::header;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt: String = env::args().collect::<Vec<_>>()[1..].join(" ");
    let key = env::var("OPENAI_API_KEY".to_string()).expect("OPENAI_API_KEY not set");

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("Authorization", ["Bearer ", &key].concat().parse()?);
    let json = &serde_json::json!({
        "model": "dall-e-3",
        "size": "1024x1024",
        "n":1,
        "prompt":prompt
        }
    );
    let client = reqwest::blocking::Client::builder().build()?;
    println!("Generating image...");
    let res = client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .json(json)
        .send()?;

    dbg!(res.text()?);
    Ok(())
}
