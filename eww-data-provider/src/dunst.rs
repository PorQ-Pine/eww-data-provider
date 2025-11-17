use serde::{Deserialize, Serialize};
use serde_json;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct DunstNotification {
    id: u64,
    summary: String,
    body: String,
    icon: String,
    appname: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DunstOutput {
    paused: bool,
    empty: bool,
    notifications: Vec<DunstNotification>,
}

#[derive(Debug, Deserialize)]
struct DunstHistoryItem {
    id: DunstData<u64>,
    summary: DunstData<String>,
    body: DunstData<String>,
    icon_path: DunstData<String>,
    appname: DunstData<String>,
}

#[derive(Debug, Deserialize)]
struct DunstData<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct DunstHistory {
    data: Vec<Vec<DunstHistoryItem>>,
}

pub async fn get_dunst_info() -> String {
    // Get pause level
    let paused_output = Command::new("dunstctl")
        .arg("get-pause-level")
        .output().await
        .unwrap();

    let paused_level = String::from_utf8_lossy(&paused_output.stdout)
        .trim()
        .parse::<u8>()
        .unwrap();
    let paused = paused_level == 1;

    // Get notifications history
    let history_output = Command::new("dunstctl").arg("history").output().await.unwrap();

    let history_json_str = String::from_utf8_lossy(&history_output.stdout);
    let dunst_history: DunstHistory = serde_json::from_str(&history_json_str).unwrap();

    let mut notifications: Vec<DunstNotification> = Vec::new();
    let mut empty = true;

    if let Some(first_level_data) = dunst_history.data.get(0) {
        if !first_level_data.is_empty() {
            empty = false;
            for item in first_level_data {
                notifications.push(DunstNotification {
                    id: item.id.data,
                    summary: item.summary.data.clone(),
                    body: item.body.data.clone(),
                    icon: item.icon_path.data.clone(),
                    appname: item.appname.data.clone(),
                });
            }
        }
    }

    let final_output = DunstOutput {
        paused,
        empty,
        notifications,
    };

    match serde_json::to_string_pretty(&final_output) {
        Ok(json) => return json,
        Err(_) => return String::from("{\"paused\": false, \"notifications\": []}"),
    };
}
