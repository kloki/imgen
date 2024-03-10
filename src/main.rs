use futures::future::try_join_all;
use indicatif::MultiProgress;
use indicatif::{ProgressBar, ProgressStyle};
use rand::distributions::{Alphanumeric, DistString};
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

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

async fn process_prompt(
    client: Client,
    headers: HeaderMap,
    prompt: String,
    pb: ProgressBar,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut name = prompt.replace(" ", "-").replace(".", "");
    name.truncate(60);
    pb.set_message(format!("🤯 Generating: {}", name));
    let res = client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .json(&ImgRequest::new(&prompt))
        .send()
        .await?;
    let payload: Payload = res.json().await?;
    let url = &payload.data[0].url;
    pb.set_message(format!("💻 Downloading: {}", name));

    let prefix = Alphanumeric.sample_string(&mut rand::thread_rng(), 5);
    let path = format!("./{}-{}.png", prefix, name);
    let mut res = client.get(url).send().await?;
    let mut dest = File::create(path.clone()).await?;

    while let Some(chunk) = res.chunk().await? {
        dest.write_all(&chunk).await?;
    }
    dest.flush().await?;

    pb.finish_with_message(format!("• {}", path));

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompts: Vec<String> = env::args().collect::<Vec<_>>()[1..].to_vec();
    let key = env::var("OPENAI_API_KEY".to_string()).expect("OPENAI_API_KEY not set");

    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template("{spinner:.green} {msg}")?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("Authorization", ["Bearer ", &key].concat().parse()?);
    let client = reqwest::Client::builder().build()?;

    let mut futures = Vec::new();
    let mut current_prompt = "".to_string();
    for prompt in prompts {
        if prompt != "." {
            current_prompt = prompt;
        }
        let pb = m.add(ProgressBar::new_spinner());
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(sty.clone());
        futures.push(process_prompt(
            client.clone(),
            headers.clone(),
            current_prompt.clone(),
            pb,
        ))
    }
    try_join_all(futures).await?;

    Ok(())
}
