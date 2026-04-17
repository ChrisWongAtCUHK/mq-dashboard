#![allow(non_snake_case)]
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// 定義與 Spring Boot 匹配的資料結構
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MqMessage {
    name: String,
    content: String,
}

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut msg_status = use_signal(|| "等待發送...".to_string());
    let mut input_text = use_signal(|| "QuickTest".to_string());

    rsx! {
        div {
            h1 { "RabbitMQ 控制台" }
            input {
                value: "{input_text}",
                oninput: move |evt| input_text.set(evt.value())
            }
            button {
                onclick: move |_| async move {
                    msg_status.set("發送中...".to_string());
                    // 呼叫伺服器端函數
                    match send_mq_rpc(input_text.read().clone()).await {
                        Ok(res) => msg_status.set(res),
                        Err(e) => msg_status.set(format!("錯誤: {}", e)),
                    }
                },
                "發送訊息"
            }
            p { "狀態: {msg_status}" }
        }
    }
}

#[server]
async fn send_mq_rpc(msg: String) -> Result<String, ServerFnError> {
    let client = reqwest::Client::new();

    // 這裡替換成你的實際網址，例如 http://localhost:5000
    let base_url = "http://localhost:5000/rabbitmq/sendTopic";

    // 建立查詢參數
    // 對應 curl 中的 ?routingKey=hk.news&name=Chris&msg=...
    let params = [("routingKey", "hk.news"), ("name", "Chris"), ("msg", &msg)];

    let response = client
        .get(base_url)
        .query(&params) // reqwest 會自動處理 URL 編碼 (URL Encoding)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("請求失敗: {}", e)))?;

    if response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        Ok(format!("發送成功: {}", body))
    } else {
        Err(ServerFnError::new(format!(
            "伺服器錯誤: {}",
            response.status()
        )))
    }
}
