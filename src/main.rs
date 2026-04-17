#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut msg_status = use_signal(|| "等待發送...".to_string());
    let mut input_text = use_signal(|| "QuickTest".to_string());

    // 將發送邏輯封裝成一個閉包，方便在按鈕和按鍵事件中重複使用
    // 這裡將參數類型顯式標註為 ()
    let mut send_msg = move |_: ()| async move {
        // 1. 在執行 await 之前，先將值取出，讓 read() 的借用立即結束
        let content = input_text.cloned();

        if content.is_empty() {
            return;
        }

        msg_status.set("發送中...".to_string());

        // 2. 這裡傳入的是已經 clone 出來的 String，不涉及 input_text 的借用
        match send_mq_rpc(content).await {
            Ok(res) => {
                msg_status.set(res);
                // 3. 這裡 set 就不會與之前的 read() 衝突
                input_text.set("".to_string()); // 發送成功後清空輸入框
            }
            Err(e) => msg_status.set(format!("錯誤: {}", e)),
        }
    };

    rsx! {
        div {
            h1 { "RabbitMQ 控制台" }
            input {
                value: "{input_text}",
                oninput: move |evt| input_text.set(evt.value()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        // 鍵盤事件需要手動 spawn
                        spawn(send_msg(()));
                    }
                }
            }
            button {
                // 直接在 onclick 裡面定義 async move 塊是最穩定的做法
                onclick: move |_| {
                    spawn(send_msg(()));
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
