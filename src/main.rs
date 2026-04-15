#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    // 0.6 版直接使用 launch 即可，它會自動處理全棧環境
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
                    if let Ok(res) = send_mq_rpc(input_text.read().clone()).await {
                        msg_status.set(res);
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
    Ok(format!("伺服器已收到: {}", msg))
}
