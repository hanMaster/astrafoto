use crate::stuff::error::{Error, Result};
use crate::stuff::hook_types::HookRoot;
use crate::stuff::timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Deserialize)]
pub enum Message {
    Text(ReceivedMessage),
    Image(ReceivedMessage),
    StateInstance(String),
    Empty,
}

impl From<HookRoot> for Message {
    fn from(m: HookRoot) -> Self {
        match m.type_webhook.as_str() {
            "incomingMessageReceived" => {
                let message_data = m.message_data.unwrap();
                let sender_data = m.sender_data.unwrap();
                match message_data.type_message.as_ref() {
                    "imageMessage" => Message::Image(ReceivedMessage {
                        chat_id: sender_data.chat_id,
                        customer_name: sender_data.sender_name,
                        message: message_data.file_message_data.unwrap().download_url,
                    }),
                    "textMessage" => Message::Text(ReceivedMessage {
                        chat_id: sender_data.chat_id,
                        customer_name: sender_data.sender_name,
                        message: message_data.text_message_data.unwrap().text_message,
                    }),
                    _ => Message::Empty,
                }
            }

            "stateInstanceChanged" => Message::StateInstance(m.state_instance.unwrap()),

            _ => Message::Empty,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReceivedMessage {
    pub chat_id: String,
    pub customer_name: String,
    pub message: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrderState {
    FilesReceiving {
        chat_id: String,
        customer_name: String,
        files: Vec<String>,
        repeats: i32,
        last_msg_time: Timestamp,
        first_prompt_sent: bool,
    },
    RaperRequested {
        chat_id: String,
        customer_name: String,
        files: Vec<String>,
        repeats: i32,
        last_msg_time: Timestamp,
    },
    SizeRequested {
        chat_id: String,
        customer_name: String,
        paper: String,
        files: Vec<String>,
        repeats: i32,
        last_msg_time: Timestamp,
    },
    SizeSelected {
        chat_id: String,
        customer_name: String,
        paper: String,
        size: String,
        price: i32,
        files: Vec<String>,
    },
}

impl OrderState {
    pub fn from_img_msg(msg: ReceivedMessage) -> OrderState {
        OrderState::FilesReceiving {
            chat_id: msg.chat_id,
            customer_name: msg.customer_name,
            files: vec![msg.message],
            repeats: 0,
            last_msg_time: Timestamp::now(),
            first_prompt_sent: false,
        }
    }

    pub fn from_txt_msg(msg: ReceivedMessage) -> OrderState {
        OrderState::FilesReceiving {
            chat_id: msg.chat_id,
            customer_name: msg.customer_name,
            files: vec![],
            repeats: 0,
            last_msg_time: Timestamp::now(),
            first_prompt_sent: true,
        }
    }

    pub fn get_chat_id(&self) -> String {
        match self {
            OrderState::FilesReceiving { chat_id, .. } => chat_id.clone(),
            OrderState::RaperRequested { chat_id, .. } => chat_id.to_string(),
            OrderState::SizeRequested { chat_id, .. } => chat_id.to_string(),
            OrderState::SizeSelected { chat_id, .. } => chat_id.to_string(),
        }
    }

    pub fn get_paper(&self) -> &str {
        match self {
            OrderState::FilesReceiving { .. } => "",
            OrderState::RaperRequested { .. } => "",
            OrderState::SizeRequested { paper, .. } => paper,
            OrderState::SizeSelected { .. } => "",
        }
    }

    pub fn last_time_sec(&self) -> u64 {
        match self {
            OrderState::FilesReceiving { last_msg_time, .. } => last_msg_time.elapsed(),
            OrderState::RaperRequested { last_msg_time, .. } => last_msg_time.elapsed(),
            OrderState::SizeRequested { last_msg_time, .. } => last_msg_time.elapsed(),
            OrderState::SizeSelected { .. } => 0,
        }
    }

    pub fn repeats(&self) -> i32 {
        match self {
            OrderState::FilesReceiving { repeats, .. } => *repeats,
            OrderState::RaperRequested { repeats, .. } => *repeats,
            OrderState::SizeRequested { repeats, .. } => *repeats,
            OrderState::SizeSelected { .. } => 0,
        }
    }

    pub fn add_image(&mut self, url: String) {
        match self {
            OrderState::FilesReceiving {
                files,
                last_msg_time,
                ..
            } => {
                files.push(url);
                *last_msg_time = Timestamp::now();
            }
            OrderState::RaperRequested {
                chat_id,
                customer_name,
                files,
                ..
            } => {
                files.push(url);
                *self = OrderState::FilesReceiving {
                    chat_id: chat_id.clone(),
                    customer_name: customer_name.clone(),
                    files: files.clone(),
                    repeats: 0,
                    last_msg_time: Timestamp::now(),
                    first_prompt_sent: false,
                }
            }
            OrderState::SizeRequested {
                chat_id,
                customer_name,
                files,
                ..
            } => {
                files.push(url);
                *self = OrderState::FilesReceiving {
                    chat_id: chat_id.clone(),
                    customer_name: customer_name.clone(),
                    files: files.clone(),
                    repeats: 0,
                    last_msg_time: Timestamp::now(),
                    first_prompt_sent: false,
                }
            }
            OrderState::SizeSelected { .. } => {}
        }
    }

    pub fn have_files(&self) -> bool {
        match self {
            OrderState::FilesReceiving { files, .. } => !files.is_empty(),
            OrderState::RaperRequested { files, .. } => !files.is_empty(),
            OrderState::SizeRequested { files, .. } => !files.is_empty(),
            OrderState::SizeSelected { files, .. } => !files.is_empty(),
        }
    }

    pub fn files_count(&self) -> usize {
        match self {
            OrderState::FilesReceiving { files, .. } => files.len(),
            OrderState::RaperRequested { files, .. } => files.len(),
            OrderState::SizeRequested { files, .. } => files.len(),
            OrderState::SizeSelected { files, .. } => files.len(),
        }
    }

    pub fn into_order_with_paper_requested(self) -> Result<OrderState> {
        match self {
            OrderState::FilesReceiving {
                chat_id,
                customer_name,
                files,
                ..
            } => Ok(OrderState::RaperRequested {
                chat_id,
                customer_name,
                files,
                repeats: 0,
                last_msg_time: Timestamp::now(),
            }),
            OrderState::RaperRequested { .. } => Err(Error::OrderWrongState),
            OrderState::SizeRequested { .. } => Err(Error::OrderWrongState),
            OrderState::SizeSelected { .. } => Err(Error::OrderWrongState),
        }
    }

    pub fn into_order_with_paper(self, paper: String) -> Result<OrderState> {
        match self {
            OrderState::RaperRequested {
                chat_id,
                customer_name,
                files,
                ..
            } => Ok(OrderState::SizeRequested {
                chat_id,
                customer_name,
                paper,
                files,
                repeats: 0,
                last_msg_time: Timestamp::now(),
            }),
            OrderState::SizeRequested { .. } => Err(Error::OrderWrongState),
            OrderState::SizeSelected { .. } => Err(Error::OrderWrongState),
            OrderState::FilesReceiving { .. } => Err(Error::OrderWrongState),
        }
    }

    pub fn into_order_with_size(self, size: String, price: i32) -> Result<OrderState> {
        match self {
            OrderState::FilesReceiving { .. } => Err(Error::OrderWrongState),
            OrderState::RaperRequested { .. } => Err(Error::OrderWrongState),
            OrderState::SizeRequested {
                chat_id,
                customer_name,
                paper,
                files,
                ..
            } => Ok(OrderState::SizeSelected {
                chat_id,
                customer_name,
                paper,
                size,
                price,
                files,
            }),
            OrderState::SizeSelected { .. } => Err(Error::OrderWrongState),
        }
    }

    pub fn requested(&mut self) {
        match self {
            OrderState::FilesReceiving {
                repeats,
                last_msg_time, ..
            } => {
                *repeats += 1;
                *last_msg_time = Timestamp::now();
            }
            OrderState::RaperRequested {
                repeats,
                last_msg_time,
                ..
            } => {
                *repeats += 1;
                *last_msg_time = Timestamp::now();
            }
            OrderState::SizeRequested {
                repeats,
                last_msg_time,
                ..
            } => {
                *repeats += 1;
                *last_msg_time = Timestamp::now();
            }
            OrderState::SizeSelected { .. } => {}
        }
    }

    pub fn set_first_prompt_has_been_requested(&mut self) {
        if let OrderState::FilesReceiving { first_prompt_sent, .. } = self {
            *first_prompt_sent = true
        }
    }
}

impl Display for OrderState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderState::FilesReceiving { .. } => {
                unimplemented!()
            }
            OrderState::RaperRequested { .. } => {
                unimplemented!()
            }
            OrderState::SizeRequested { .. } => {
                unimplemented!()
            }
            OrderState::SizeSelected {
                chat_id,
                customer_name,
                paper,
                size,
                files,
                ..
            } => {
                let phone = chat_id.split('@').collect::<Vec<&str>>()[0];
                write!(
                    f,
                    "Телефон: {phone}\nИмя: {}\nТип бумаги: {}\nРазмер: {}\nФайлы: {:?}",
                    customer_name, paper, size, files
                )
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OrderMessage {
    pub phone: String,
    pub name: String,
    pub paper_type: String,
    pub paper_size: String,
    pub price: i32,
    pub files: Vec<String>,
}

impl From<OrderState> for OrderMessage {
    fn from(order: OrderState) -> Self {
        match order {
            OrderState::FilesReceiving { .. } => {
                unreachable!()
            }
            OrderState::RaperRequested { .. } => {
                unreachable!()
            }
            OrderState::SizeRequested { .. } => {
                unreachable!()
            }
            OrderState::SizeSelected {
                chat_id,
                customer_name,
                paper,
                size,
                price,
                files,
                ..
            } => {
                let phone = chat_id.split('@').collect::<Vec<&str>>()[0];
                Self {
                    phone: phone.to_string(),
                    name: customer_name,
                    paper_type: paper,
                    paper_size: size,
                    price,
                    files,
                }
            }
        }
    }
}
