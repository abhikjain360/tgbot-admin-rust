use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::command::BotCommand;

use std::fs::{File, OpenOptions};
use std::io::prelude::*;

use std::net::TcpListener;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "show banned words.")]
    List,
    #[command(description = "add new banned word.")]
    Add(String),
    #[command(description = "add new admin.")]
    AddAdmin(String),
}

async fn answer(cx: UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    match command {
        Command::Help => {
            cx.answer_str(Command::descriptions()).await?;
        },
        Command::List => {
            let mut words = String::new();
            for word in get_words() {
                words += &word;
                words += "\n";
            }
            cx.answer_str(words).await?;
        }
        Command::Add(word) => {
            let id = match cx.update.from() {
                Some(user) => user.id,
                None => {
                    cx.answer_str("only current admins can change list").await?;
                    return Ok(());
                }
            };
            if check_admin(id) {
                if add_word(word.clone()).is_err() {
                    cx.answer_str("unable to add word").await?;
                } else {
                    cx.answer_str(format!("word \"{}\" added successfully", word))
                        .await?;
                }
            }
        }
        Command::AddAdmin(user_id) => {
            let id = match cx.update.from() {
                Some(user) => user.id,
                None => {
                    cx.answer_str("only current admins can change list").await?;
                    return Ok(());
                }
            };
            if check_admin(id) {
                if add_admin(user_id.clone()).is_err() {
                    cx.answer_str("unable to add user").await?;
                } else {
                    cx.answer_str(format!("word \"{}\" added successfully", user_id))
                        .await?;
                }
            }

        }
    }

    Ok(())
}

fn add_word(mut word: String) -> Result<(), String> {
    word += "\n";
    let file = OpenOptions::new().append(true).open("./banned_words");
    match file {
        Ok(mut file) => {
            if file
                .write_all(&word.to_lowercase().into_bytes()[..])
                .is_err()
            {
                println!("unable to write to file of banned words!!");
                return Err(String::from("unable to write to file of banned words!!"));
            }
        }
        Err(_) => {
            println!("unable to open file of banned words!!");
            return Err(String::from("unable to open file of banned words!!"));
        }
    }
    Ok(())
}

fn get_words() -> Vec<String> {
    let mut words = vec![];

    let file = File::open("./banned_words");
    let mut contents = String::new();
    match file {
        Ok(mut file) => {
            file.read_to_string(&mut contents).unwrap();
            words = contents
                .split_whitespace()
                .map(|word_str| word_str.to_string())
                .collect();
        }
        Err(_) => {
            log::info!("unable to open file of banned words!!");
        }
    }

    words
}

fn check_admin(id: i32) -> bool {
    let file = File::open("./admins");
    let mut contents = String::new();
    let id = id.to_string();
    match file {
        Ok(mut file) => {
            file.read_to_string(&mut contents).unwrap();
            for word in contents.split_whitespace() {
                if word == id {
                    return true;
                }
            }
            false
        }
        Err(_) => {
            log::info!("unable to open file of admins!!");
            false
        }
    }
}

fn add_admin(id: String) -> Result<(), String> {
    let file = OpenOptions::new().append(true).open("./admins");
    match file {
        Ok(mut file) => {
            if file
                .write_all(&id.to_lowercase().into_bytes()[..])
                .is_err()
            {
                println!("unable to write to file of admins!!");
                return Err(String::from("unable to write to file of admins!!"));
            }
        }
        Err(_) => {
            println!("unable to open file of admins!!");
            return Err(String::from("unable to open file of admins!!"));
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    log::info!("Starting kaka_testbot ...");

    std::thread::spawn(|| {
        let port = std::env::var("PORT").unwrap_or(String::from("8080"));
        let url = String::from("0.0.0.0:") + &port;

        let listener = TcpListener::bind(url).expect("unable to bind to address!");
        for stream in listener.incoming() {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            let mut stream = stream.expect("something wrong with streams!");
            stream
                .write(response.as_bytes())
                .expect("unable to write to incoming stream");
            stream.flush().expect("unable to flush to stream");
        }
    });

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            let words = Box::new(get_words());
            rx.map(move |m| (m, words.clone())).for_each_concurrent(None, |(message, test)| async move {
                if let Some(msg) = message.update.text() {
                    if let Ok(command) = Command::parse(msg, "pae_group_bot") {
                        answer(message, command).await.unwrap();
                    } else {
                        let words = test.clone();
                        for word in words.iter() {
                            if msg.to_lowercase().contains(word) {
                                match message.update.from() {
                                    Some(user) => {
                                        log::info!(
                                            "bad word located from user : {}",
                                            user.id
                                        );
                                        match message.bot.kick_chat_member(message.chat_id(), user.id).send().await {
                                            Ok(_) => {
                                                log::info!("kicked : {}", user.id);
                                            },
                                            Err(_) => {
                                                std::mem::drop(word);
                                                if message
                                                    .bot
                                                        .send_message(
                                                            message.chat_id(),
                                                            format!("Unable to kick member using banned words.\nPossible I don't have the permissions")
                                                        )
                                                        .send()
                                                        .await
                                                        .is_err() {
                                                            log::info!("unable to send messages!!");
                                                }
                                            },
                                        }
                                    },
                                    None => {

                                    }
                                }
                            }
                        }
                    }

                } else if let Some(users) = message.update.new_chat_members() {
                    let mut msg = String::from("\nHey ");
                    for user in users {
                        msg += "@";
                        match &user.username {
                            Some(name) => msg += name,
                            None => {
                                msg += &user.id.to_string();
                            }
                        };
                        msg += " ";
                    }

                    msg += ", welcome !!";

                    let markup = InlineKeyboardMarkup::default()
                        .append_row(vec![InlineKeyboardButton::url(
                                String::from("List to all the Blog Posts"),
                                String::from("http://link1")
                                )])
                        .append_row(vec![InlineKeyboardButton::url(
                                String::from("Problem Solving Group"),
                                String::from("http://link2")
                                ),
                                InlineKeyboardButton::url(
                                String::from("Applying Abroad Group"),
                                String::from("http://link3")
                                )])
                        .append_row(vec![InlineKeyboardButton::url(
                                String::from("Pop-Sci Group"),
                                String::from("http://link4")
                                ),
                                InlineKeyboardButton::url(
                                String::from("Fun Group"),
                                String::from("http://link5")
                                )])
                        .append_row(vec![InlineKeyboardButton::url(
                                String::from("Notes & Material"),
                                String::from("link6")
                                )])
                        .append_row(vec![InlineKeyboardButton::url(
                                String::from("FAQs - Must Read!!"),
                                String::from("link7")
                                )]);
                    if message.bot.send_message(message.update.chat_id(), msg).reply_markup(markup).send().await.is_err() {
                        log::info!("unable to send welcome messages!!");
                    }
                    msg = String::from("\nexit section message\n");
                    if message.bot.send_message(message.update.chat_id(), msg).send().await.is_err() {
                        log::info!("unable to send welcome messages!!");
                    } else {
                        log::info!("Message sent!!");
                    }
                }
            })
        })
        .dispatch()
        .await;
}
