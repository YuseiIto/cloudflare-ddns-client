# cloudflare-ddns-client

DDNS client for cloudflare API v4.

## Requirements
- Cargo

## Instalaltion
You can install this by cargo install.

``` zsh
cargo install --git https://github.com/YuseiIto/cloudflare-ddns-client
```

## Configuration
Place following TOML configuration file at some location.

```toml
zone_id=<CLOUDFLARE_ZONE_ID>
api_key=<CLOUDFLARE_API_KEY>
email=<CLOUDFLARE LOGIN EMAIL>
record_name=<RECORD_NAME>
```


## Cron setup
Add following to execute update every fifth minutes.

```crontab
*/5 * * * * cloudflare-ddns-client /path/to/config.toml
```

## Licence
Licenced under Apache Licence 2.0.

