use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
    let submit = ctx.link().callback(|_| Msg::SubmitMessage);
    html! {
        <div class="flex w-screen h-screen font-sans bg-gray-100 text-gray-800">
            // Sidebar - Users
            <div class="w-64 bg-gray-800 text-white border-r border-gray-700 p-4">
                <h2 class="text-lg font-semibold mb-4">{"👥 Online Users"}</h2>
                {
                    self.users.iter().map(|u| {
                        html!{
                            <div class="flex items-center mb-3 hover:bg-gray-700 p-2 rounded-lg transition">
                                <img class="w-10 h-10 rounded-full mr-3" src={u.avatar.clone()} alt="avatar" />
                                <div class="text-sm font-medium">{u.name.clone()}</div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>

            // Chat Area
            <div class="flex flex-col flex-grow">
                // Header
                <div class="h-16 flex items-center px-6 border-b border-gray-300 bg-gray-100 shadow-sm">
                    <h1 class="text-xl font-semibold">{"💬 Chat Room"}</h1>
                </div>

                // Messages
                <div class="flex-grow overflow-y-auto p-6 space-y-4 bg-gray-100">
                    {
                        self.messages.iter().map(|m| {
                            // Safe user lookup with fallback
                            let avatar = self.users.iter()
                                .find(|u| u.name == m.from)
                                .map(|u| u.avatar.clone())
                                .unwrap_or_else(|| {
                                    format!("https://avatars.dicebear.com/api/initials/{}.svg", m.from)
                                });

                            let name = m.from.clone();
                            let message = m.message.clone();

                            html! {
                                <div class="flex items-start space-x-3 bg-white p-4 rounded-lg shadow w-fit max-w-xl">
                                    <img class="w-10 h-10 rounded-full" src={avatar} alt="avatar"/>
                                    <div>
                                        <div class="font-semibold text-sm mb-1">{name}</div>
                                        <div class="text-sm text-gray-700">
                                            {
                                                if message.ends_with(".gif") {
                                                    html! { <img class="mt-2 rounded" src={message} /> }
                                                } else {
                                                    html! { <p>{message}</p> }
                                                }
                                            }
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>

                // Input Box
                <div class="h-16 bg-white border-t border-gray-200 flex items-center px-4">
                    <input
                        ref={self.chat_input.clone()}
                        type="text"
                        placeholder="Type your message..."
                        class="flex-grow bg-gray-100 rounded-full px-4 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500"
                        name="message"
                        required=true
                    />
                    <button onclick={submit} class="ml-3 bg-blue-600 hover:bg-blue-500 text-white rounded-full w-10 h-10 flex justify-center items-center shadow">
                        <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 fill-current">
                            <path d="M0 0h24v24H0z" fill="none"></path>
                            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}


}