
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

pub const DEFAULT_USER_ID_STR: &'static str = "337229462"; // Parviz Sadesi
pub const DEFAULT_GROUP_ID_STR: &'static str = "-222927743"; // Beercan

#[derive(Clone, Debug, StructOpt)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
#[structopt(setting = AppSettings::AllowLeadingHyphen)]
pub struct CliArgs {
    /// user id to remind about vaccination
    #[structopt(short = "u", long = "delete-recover-user-id", default_value = DEFAULT_USER_ID_STR)]
    user_id: Integer,

    /// chat id to use
    #[structopt(short = "g", long = "delete-recover-group-id", default_value = DEFAULT_GROUP_ID_STR)]
    group_id: Integer,
}

#[derive(Debug)]
pub enum Error {
    TelegramApiSend(telegram_bot::Error),
}

pub struct DeleteRecover {
    user_id: UserId,
    group_id: GroupId,
}

impl DeleteRecover {
    pub fn new(cli_args: &CliArgs) -> Result<DeleteRecover, Error> {
        Ok(DeleteRecover {
            user_id: cli_args.user_id.into(),
            group_id: cli_args.group_id.into(),
        })
    }

    pub async fn process(&mut self, update: &Update, api: &Api) -> Result<(), Error> {
        match &update.kind {
            UpdateKind::Message(message) =>
                match message {
                    Message {
                        from: User { id: user_id, .. },
                        chat: MessageChat::Group(Group { id: chat_id, .. }),
                        kind: MessageKind::Text { data, .. },
                        ..
                    } if user_id == &self.user_id && chat_id == &self.group_id => {
                        // let _message_or_channel_post = api.send(message.text_reply(reply_phrase)).await
                        //     .map_err(Error::TelegramApiSend)?;
                    },
                    other_message =>
                        log::debug!("other message kind: {:?}", other_message),
                },
            other_update =>
                log::debug!("other update kind: {:?}", other_update),
        }
        Ok(())
    }
}
