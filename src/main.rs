use std::{env, time::Duration};

use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::distributions::{Alphanumeric, DistString};
use reqwest::{header::HeaderMap, Client};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};

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

async fn process_prompt(client: Client, headers: HeaderMap, prompt: String, pb: ProgressBar) {
    let mut name = prompt.clone();
    name.truncate(60);
    pb.set_message(format!("[generating] {}", name));
    let res = match client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .json(&ImgRequest::new(&prompt))
        .send()
        .await
    {
        Ok(s) => s,
        Err(e) => {
            pb.finish_with_message(format!("[error-gen] {}", e));
            return;
        }
    };
    if res.status() != 200 {
        pb.finish_with_message(format!("[error-gen] {}", res.status()));
        return;
    }
    let payload: Payload = match res.json().await {
        Ok(s) => s,
        Err(_) => {
            pb.finish_with_message("[error-gen] failed to parse json");
            return;
        }
    };
    let url = &payload.data[0].url;
    pb.set_message(format!("[downloading] {}", name));

    let unique = Alphanumeric.sample_string(&mut rand::thread_rng(), 5);
    name = name.replace(' ', "-");
    name = name
        .chars()
        .filter(|x| x.is_alphanumeric() || *x == '-')
        .collect::<String>();
    let path = format!("./{}-{}.png", name, unique);
    let mut res = match client.get(url).send().await {
        Ok(s) => s,
        Err(e) => {
            pb.finish_with_message(format!("[error-down] {}", e));
            return;
        }
    };

    if res.status() != 200 {
        pb.finish_with_message(format!("[error-down] status_code {}", res.status()));
        return;
    }
    let mut dest = match File::create(path.clone()).await {
        Ok(s) => s,
        Err(e) => {
            pb.finish_with_message(format!("[error-down] {}", e));
            return;
        }
    };

    while let Some(chunk) = match res.chunk().await {
        Ok(s) => s,
        Err(e) => {
            pb.finish_with_message(format!("[error-down] {}", e));
            return;
        }
    } {
        if let Err(e) = dest.write_all(&chunk).await {
            pb.finish_with_message(format!("[error-down] {}", e));
            return;
        }
    }
    if let Err(e) = dest.flush().await {
        pb.finish_with_message(format!("[error-down] {}", e));
        return;
    }

    pb.finish_with_message(format!("[done] {}", path));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompts: Vec<String> = env::args().collect::<Vec<_>>()[1..].to_vec();
    let key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

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
    join_all(futures).await;

    Ok(())
}
