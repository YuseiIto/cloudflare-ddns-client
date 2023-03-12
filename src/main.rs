use anyhow::{Context, Result};
mod model;
use model::*;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().len() != 2 {
        println!("Usage: {} <config_file>", std::env::args().nth(0).unwrap());
        return Ok(());
    }

    let config_file =
        std::fs::read_to_string(std::env::args().nth(1).unwrap()).with_context(|| {
            format!(
                "Failed to read config file {}",
                std::env::args().nth(1).unwrap()
            )
        })?;

    let config: Config = toml::from_str(&config_file).with_context(|| {
        format!(
            "Failed to parse config file {}",
            std::env::args().nth(1).unwrap()
        )
    })?;

    println!("Updating Cloudflare DNS record");

    let my_ip = fetch_my_ip()
        .await
        .expect("Failed to fetch my IP")
        .trim()
        .to_string();
    println!("My IP: {}", my_ip);

    let records = list_records(&config)
        .await
        .with_context(|| "Failed to list records.".to_string())?;
    let record = records.iter().find(|r| r.name == config.record_name);

    let updated_record = Record {
        content: my_ip.to_string(),
        name: config.record_name.to_string(),
        type_: "A".to_string(),
        comment: Some(format!("DDNS Last update at {}", chrono::Utc::now())),
        id: None, // Ignored
    };

    match record {
        None => {
            println!("Record does not exist. Creating new one.");
            create_record(&updated_record, &config).await?;
        }
        Some(_) => {
            println!("Record already exists!");
            if record.unwrap().content == updated_record.content {
                println!("Record is already up to date. Nothing to do.");
                return Ok(());
            }
            let record_id = record.unwrap().id.as_ref().unwrap();
            put_record(record_id, &updated_record, &config).await?;
        }
    }

    Ok(())
}

async fn list_records(config: &Config) -> Result<Vec<Record>> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            config.zone_id
        ))
        .header("X-Auth-Email", &config.email)
        .header("X-Auth-Key", &config.api_key)
        .send()
        .await?;

    let txt = res.text().await?;

    let res = serde_json::from_str::<ListRequestResponse>(&txt)?;

    if res.messages.is_some() {
        let msgs = res.messages.unwrap();
        if !msgs.is_empty() {
            println!("Cloudflare returned messages: {:?}", msgs);
        }
    }

    match res.success {
        true => Ok(res.result.unwrap()),
        false => Err(anyhow::anyhow!("Cloudflare API error. {:?}", res.errors)),
    }
}

async fn fetch_my_ip() -> Result<String> {
    let res = reqwest::get("https://api.ipify.org/").await?.text().await?;
    Ok(res)
}

async fn put_record(record_id: &str, record: &Record, config: &Config) -> Result<()> {
    println!("Updating record...");
    let client = reqwest::Client::new();
    let res: WriteRequestResponse = client
        .put(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            config.zone_id, record_id
        ))
        .header("X-Auth-Email", &config.email)
        .header("X-Auth-Key", &config.api_key)
        .json(&record)
        .send()
        .await?
        .json()
        .await?;

    if res.messages.is_some() {
        let msgs = res.messages.unwrap();
        if !msgs.is_empty() {
            println!("Cloudflare returned messages: {:?}", msgs);
        }
    }

    match res.success {
        true => {
            println!("Record updated!");
            Ok(())
        }
        false => Err(anyhow::anyhow!(
            "Cloudflare API error while updating record"
        )),
    }
}

async fn create_record(record: &Record, config: &Config) -> Result<()> {
    println!("Creating record...");
    let client = reqwest::Client::new();
    let res: WriteRequestResponse = client
        .post(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            &config.zone_id
        ))
        .header("X-Auth-Email", &config.email)
        .header("X-Auth-Key", &config.api_key)
        .json(record)
        .send()
        .await?
        .json()
        .await?;

    if res.messages.is_some() {
        let msgs = res.messages.unwrap();
        if !msgs.is_empty() {
            println!("Cloudflare returned messages: {:?}", msgs);
        }
    }

    match res.success {
        true => {
            println!("Record created!");
            Ok(())
        }
        false => Err(anyhow::anyhow!(
            "Cloudflare API error while creating record. Reason:{:?}",
            res.errors
        )),
    }
}
