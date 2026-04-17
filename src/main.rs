#![allow(non_snake_case)]
use chrono::Local; // 用於獲取本地時間
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

// 定義歷史紀錄的資料結構
#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct MessageItem {
    content: String,
    time: String,
}

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut msg_status = use_signal(|| "等待發送...".to_string());
    let mut input_text = use_signal(|| "QuickTest".to_string());
    let mut is_loading = use_signal(|| false); // 這裡定義一個新的 state 來追蹤是否正在發送訊息
    let mut history = use_signal(|| Vec::<MessageItem>::new()); // 先初始化為空向量

    // 使用 use_effect 在組件掛載後（僅在瀏覽器端執行）讀取資料
    use_effect(move || {
        if let Ok(saved) = LocalStorage::get::<Vec<MessageItem>>("mq_history") {
            history.set(saved);
        }
    });

    let send_msg = move |_: ()| async move {
        // 在執行 await 之前，先將值取出，讓 read() 的借用立即結束
        let text = input_text.cloned();

        // 如果正在發送中或輸入框為空，則不執行
        if is_loading() || text.is_empty() {
            return;
        }

        // 開始發送：設定 loading 為 true，並更新狀態文字
        is_loading.set(true);
        msg_status.set("發送中...".to_string());

        // 先 Clone 一份複本給 RPC
        let rpc_content = text.clone();

        match send_mq_rpc(rpc_content).await {
            Ok(res) => {
                msg_status.set(res);
                input_text.set("".to_string()); // 發送成功後清空輸入框

                // 建立帶有時間戳記的物件
                let new_item = MessageItem {
                    content: text,
                    time: Local::now().format("%H:%M:%S").to_string(), // 格式如 14:30:05
                };

                let mut h = history.write(); // 發送成功，將訊息加入歷史清單的最前面
                h.insert(0, new_item);
                if h.len() > 5 {
                    h.pop();
                } // 只保留最近 5 筆
                  // 使用 cloned() 獲取資料複本進行存檔
                let _ = LocalStorage::set("mq_history", h.clone());
            }
            Err(e) => {
                // 失敗則保留輸入內容，僅更新狀態文字
                msg_status.set(format!("發送失敗: {}", e));
            }
        }

        // 結束發送：無論成功或失敗，都要把 loading 設回 false
        is_loading.set(false);
    };

    rsx! {
        document::Script { src: "https://cdn.tailwindcss.com" }
        div {
            // 修正 1: 去掉所有中間層的 min-h-screen，只在最外層保留一次
            // 修正 2: 確保寬度撐滿 w-full，並在手機版預設 flex-col (上下排)
            class: "min-h-screen w-full bg-gray-100 flex flex-col lg:flex-row items-center lg:items-start justify-center p-6 lg:p-12 gap-8",


            // 主控制台卡片
            div {
                // 修正 3: 使用 flex-none 確保卡片不會被擠壓，w-full 確保寬度
                class: "w-full max-w-md bg-white rounded-2xl shadow-xl p-8 space-y-6 flex-none",
                h1 { class: "text-2xl font-bold text-gray-800 text-center", "RabbitMQ 控制台" }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium text-gray-600", "訊息內容" }
                        input {
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition-all disabled:bg-gray-50",
                            placeholder: "輸入訊息...",
                            disabled: is_loading(),  // 發送時也可以禁用輸入框，防止修改
                            value: "{input_text}",
                            oninput: move |evt| input_text.set(evt.value()),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter {
                                    // 鍵盤事件需要手動 spawn
                                    spawn(send_msg(()));
                                }
                            }
                        }
                }
                button {
                    // 動態樣式：發送時變灰色，平常是藍色
                    class: format!(
                        "w-full py-3 rounded-lg font-semibold text-white transition-all {} ",
                        if is_loading() { "bg-gray-400 cursor-not-allowed" } else { "bg-blue-600 hover:bg-blue-700 active:scale-95" }
                    ),
                    disabled: is_loading(), // 根據 is_loading 禁用按鈕
                    onclick: move |_| async move {
                        send_msg(()).await;
                    },
                    if is_loading() { "處理中..." } else { "發送訊息" }
                }
                // 狀態顯示區
                div {
                    class: format!(
                        "p-4 rounded-xl text-sm font-medium transition-all border shadow-inner {}",
                        if msg_status().contains("錯誤") || msg_status().contains("失敗") {
                            "bg-red-50 text-red-600 border-red-200"
                        } else if msg_status().contains("成功") {
                            "bg-emerald-50 text-emerald-600 border-emerald-200"
                        } else {
                            "bg-blue-50 text-blue-600 border-blue-200"
                        }
                    ),
                    // 顯示內容
                    span { class: "mr-2",
                        if msg_status().contains("成功") { "🎉" }
                        else if msg_status().contains("錯誤") || msg_status().contains("失敗") { "⚠️" }
                        else { "ℹ️" }
                    }
                    "{msg_status}"
                }
            }
            // 歷史紀錄卡片放在外面，與主卡片同級
            // 歷史紀錄顯示部分
            if !history.read().is_empty() {
                div { class: "w-full max-w-md bg-white rounded-xl shadow-md p-6 animate-fade-in flex-none",
                    h2 { class: "text-sm font-bold text-gray-500 uppercase tracking-wider mb-4", "最近發送紀錄" }
                    ul { class: "divide-y divide-gray-100",
                        for (i, item) in history.read().iter().enumerate() {
                            li { key: "{i}", class: "py-3 flex flex-col gap-1",
                                div { class: "flex items-center justify-between",
                                    span { class: "text-gray-800 font-medium break-words", "{item.content}" }
                                    span { class: "shrink-0 text-[10px] bg-green-100 text-green-600 px-2 py-0.5 rounded", "成功" }
                                }
                                // 顯示時間戳記
                                span { class: "text-[10px] text-gray-400 font-mono", "🕒 {item.time}" }
                            }
                        }
                    }
                }
            }
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
        .map_err(|e| {
            // 處理連線層級的錯誤（例如：伺服器沒開、網址打錯）
            if e.is_connect() {
                ServerFnError::new("無法連線至 Spring Boot 伺服器，請檢查後端是否啟動。")
            } else if e.is_timeout() {
                ServerFnError::new("伺服器回應超時。")
            } else {
                ServerFnError::new(format!("網路請求異常: {}", e))
            }
        })?;

    match response.status() {
        s if s.is_success() => Ok(format!(
            "成功！伺服器回應: {}",
            response.text().await.unwrap_or_default()
        )),
        reqwest::StatusCode::NOT_FOUND => Err(ServerFnError::new(
            "找不到該 API 路徑 (404)，請檢查 Spring Boot 路由。",
        )),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR => Err(ServerFnError::new(
            "Spring Boot 內部錯誤 (500)，可能是 RabbitMQ 連線失敗。",
        )),
        other => Err(ServerFnError::new(format!(
            "伺服器回傳未預期狀態: {}",
            other
        ))),
    }
}
