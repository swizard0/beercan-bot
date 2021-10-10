
use structopt::{
    clap::{
        AppSettings,
    },
    StructOpt,
};

use telegram_bot::{
    types::{
        UserId,
        GroupId,
        Integer,
    },
    Api,
    User,
    Group,
    Update,
    Message,
    UpdateKind,
    MessageChat,
    MessageKind,
    CanReplySendMessage,
};

#[derive(Clone, Debug, StructOpt)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
#[structopt(setting = AppSettings::AllowLeadingHyphen)]
pub struct CliArgs {
    /// user id to remind about vaccination
    #[structopt(short = "u", long = "vaccine-reminder-user-id")]
    user_id: Integer,

    /// chat id to use
    #[structopt(short = "g", long = "vaccine-reminder-group-id")]
    group_id: Integer,
}

#[derive(Debug)]
pub enum Error {
    TelegramApiSend(telegram_bot::Error),
}

pub struct VaccineReminder {
    user_id: UserId,
    group_id: GroupId,
}

impl VaccineReminder {
    pub fn new(cli_args: &CliArgs) -> Result<VaccineReminder, Error> {
        Ok(VaccineReminder {
            user_id: cli_args.user_id.into(),
            group_id: cli_args.group_id.into(),
        })
    }

    pub async fn process(&self, update: &Update, api: &Api) -> Result<(), Error> {
        match &update.kind {
            UpdateKind::Message(message) =>
                match message {
                    Message {
                        from: User { id: user_id, .. },
                        chat: MessageChat::Group(Group { id: chat_id, .. }),
                        kind: MessageKind::Text { data, .. },
                        ..
                    } if user_id == &self.user_id && chat_id == &self.group_id && is_question(data) => {

                        // if let UpdateKind::Message(message) = &update.kind {
                        //     if let MessageKind::Text { data, .. } = &message.kind {

                        println!("<{}>: {}", &message.from.first_name, data);
                        println!("{:?}", update);

                        let _message_or_channel_post = api.send(message.text_reply("Эй, как насчёт укола в жопу?")).await
                            .map_err(Error::TelegramApiSend)?;
                    },
                    other_message =>
                        println!("other message kind: {:?}", other_message),
                },
            other_update =>
                println!("other update kind: {:?}", other_update),
        }
        Ok(())
    }
}

fn is_question(message: &str) -> bool {
    for ch in message.chars().rev() {
        if ch == '?' {
            return true;
        }
        if ch.is_alphanumeric() {
            break;
        }
    }

    false
}
