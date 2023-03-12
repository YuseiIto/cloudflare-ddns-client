use anyhow::{Context, Result};
mod model;
use model::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Updating Cloudflare DNS record");

    let zone_id = std::env::var("CLOUDFLARE_ZONE_ID").expect("CLOUDFLARE_ZONE_ID not set");
    let api_key = std::env::var("CLOUDFLARE_API_KEY").expect("CLOUDFLARE_API_KEY not set");
    let email = std::env::var("CLOUDFLARE_EMAIL").expect("CLOUDFLARE_EMAIL not set");
    let record_name =
        std::env::var("CLOUDFLARE_RECORD_NAME").expect("CLOUDFLARE_RECORD_NAME not set");

    let my_ip = fetch_my_ip()
        .await
        .expect("Failed to fetch my IP")
        .trim()
        .to_string();
    println!("My IP: {}", my_ip);

    let records = list_records(&zone_id, &email, &api_key)
        .await
        .with_context(|| "Failed to list records.".to_string())?;
    let record = records.iter().find(|r| r.name == record_name);

    let updated_record = Record {
        content: my_ip.to_string(),
        name: record_name.to_string(),
        type_: "A".to_string(),
        comment: Some(format!("DDNS Last update at {}", chrono::Utc::now())),
        id: None, // Ignored
    };

    match record {
        None => {
            println!("Record does not exist. Creating new one.");
            create_record(&updated_record, &zone_id, &api_key, &email).await?;
        }
        Some(_) => {
            println!("Record already exists!");
            if record.unwrap().content == updated_record.content {
                println!("Record is already up to date. Nothing to do.");
                return Ok(());
            }
            let record_id = record.unwrap().id.as_ref().unwrap();
            put_record(&zone_id, record_id, &updated_record, &api_key, &email).await?;
        }
    }

    Ok(())
}

async fn list_records(zone_id: &str, email: &str, api_key: &str) -> Result<Vec<Record>> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        ))
        .header("X-Auth-Email", email)
        .header("X-Auth-Key", api_key)
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

async fn put_record(
    zone_id: &str,
    record_id: &str,
    record: &Record,
    api_key: &str,
    email: &str,
) -> Result<()> {
    println!("Updating record...");
    let client = reqwest::Client::new();
    let res: WriteRequestResponse = client
        .put(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        ))
        .header("X-Auth-Email", email)
        .header("X-Auth-Key", api_key)
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

async fn create_record(record: &Record, zone_id: &str, api_key: &str, email: &str) -> Result<()> {
    println!("Creating record...");
    let client = reqwest::Client::new();
    let res: WriteRequestResponse = client
        .post(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        ))
        .header("X-Auth-Email", email)
        .header("X-Auth-Key", api_key)
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
