use indicatif::{ProgressBar, ProgressStyle};
use rand::distributions::{Alphanumeric, DistString};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    data: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImgRequest<'a> {
    model: &'a str,
    size: &'a str,
    n: usize,
    prompt: &'a str,
}

impl ImgRequest<'_> {
    fn new(prompt: &str) -> ImgRequest {
        ImgRequest {
            model: "dall-e-3",
            size: "1024x1024",
            n: 1,
            prompt,
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt: String = env::args().collect::<Vec<_>>()[1..].join(" ");
    let key = env::var("OPENAI_API_KEY".to_string()).expect("OPENAI_API_KEY not set");

    let sty = ProgressStyle::with_template("{spinner:.green} {msg}")?;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(sty);

    pb.set_message("🤯 Generating");
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("Authorization", ["Bearer ", &key].concat().parse()?);
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .json(&ImgRequest::new(&prompt))
        .send()?;
    let payload: Payload = res.json()?;
    let url = &payload.data[0].url;

    pb.set_message("💻 Downloading");
    let prefix = Alphanumeric.sample_string(&mut rand::thread_rng(), 5);
    let mut name = prompt.replace(" ", "-").replace(".", "");
    name.truncate(60);
    let path = format!("./{}-{}.png", prefix, name);
    let mut file = std::fs::File::create(&path)?;
    client.get(url).send()?.copy_to(&mut file)?;

    pb.finish_with_message(format!(" 🎉 {}", path));

    Ok(())
}
