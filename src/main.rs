#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut msg_status = use_signal(|| "等待發送...".to_string());
    let mut input_text = use_signal(|| "QuickTest".to_string());
    let mut is_loading = use_signal(|| false); // 這裡定義一個新的 state 來追蹤是否正在發送訊息

    // 將發送邏輯封裝成一個閉包，方便在按鈕和按鍵事件中重複使用
    // 這裡將參數類型顯式標註為 ()
    let send_msg = move |_: ()| async move {
        // 在執行 await 之前，先將值取出，讓 read() 的借用立即結束
        let content = input_text.cloned();

        // 如果正在發送中或輸入框為空，則不執行
        if is_loading() || content.is_empty() {
            return;
        }

        // 開始發送：設定 loading 為 true，並更新狀態文字
        is_loading.set(true);
        msg_status.set("發送中...".to_string());

        // 這裡傳入的是已經 clone 出來的 String，不涉及 input_text 的借用
        match send_mq_rpc(content).await {
            Ok(res) => {
                msg_status.set(res);
                input_text.set("".to_string()); // 發送成功後清空輸入框
            }
            Err(e) => msg_status.set(format!("錯誤: {}", e)),
        }

        // 結束發送：無論成功或失敗，都要把 loading 設回 false
        is_loading.set(false);
    };

    rsx! {
        div {
            h1 { "RabbitMQ 控制台" }
            input {
                // 發送時也可以禁用輸入框，防止修改
                disabled: is_loading(),
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
                // 根據 is_loading 禁用按鈕
                disabled: is_loading(),
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
