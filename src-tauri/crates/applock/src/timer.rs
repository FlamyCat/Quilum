use chrono::{DateTime, Utc};

pub async fn wait_until(end_time: DateTime<Utc>) {
    let now = Utc::now();
    if end_time > now {
        tokio::time::sleep((end_time - now).to_std().unwrap()).await;
    }
}
