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
    let mut history = use_signal(|| Vec::<String>::new()); // 新增歷史紀錄清單，儲存最近 5 筆
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

        // 先 Clone 一份複本給 RPC
        let rpc_content = content.clone();

        match send_mq_rpc(rpc_content).await {
            Ok(res) => {
                msg_status.set(res);
                input_text.set("".to_string()); // 發送成功後清空輸入框

                let mut h = history.write(); // 發送成功，將訊息加入歷史清單的最前面
                h.insert(0, content); // use of moved value: `content` value used here after move
                if h.len() > 5 {
                    h.pop();
                } // 只保留最近 5 筆
            }
            Err(e) => msg_status.set(format!("錯誤: {}", e)),
        }

        // 結束發送：無論成功或失敗，都要把 loading 設回 false
        is_loading.set(false);
    };

    rsx! {
        document::Script { src: "https://cdn.tailwindcss.com" }
        div {
            div {
                class: "min-h-screen bg-gray-100 flex items-center justify-center p-4",
                div { class: "max-w-md w-full bg-white rounded-xl shadow-lg p-8 space-y-6",
                    h1 { class: "text-2xl font-bold text-gray-800 text-center", "RabbitMQ 控制台" }

                    div {
                        class: "space-y-2",
                            label { class: "text-sm font-medium text-gray-600", "訊息內容" }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all disabled:bg-gray-50 disabled:text-gray-400",
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
                            if is_loading() { "bg-gray-400 cursor-not-allowed" } else { "bg-blue-600 hover:bg-blue-700 active:transform active:scale-95" }
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
                            "p-4 rounded-lg text-sm font-mono {}",
                            if msg_status().contains("錯誤") { "bg-red-50 text-red-600" } else { "bg-blue-50 text-blue-600" }
                        ),
                        "狀態: {msg_status}"
                    }
                }
                // 歷史紀錄卡片放在外面，與主卡片同級
                if !history.read().is_empty() {
                    div { class: "bg-white rounded-xl shadow-md p-6 animate-fade-in",
                        h2 { class: "text-sm font-bold text-gray-500 uppercase tracking-wider mb-4", "最近發送紀錄" }
                        ul { class: "divide-y divide-gray-100",
                            for (i, msg) in history.read().iter().enumerate() {
                                li { key: "{i}", class: "py-3 flex items-center justify-between",
                                    span { class: "text-gray-700 font-medium", "{msg}" }
                                    span { class: "text-xs bg-green-100 text-green-600 px-2 py-1 rounded", "成功" }
                                }
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
