#![forbid(unsafe_code)]

use futures::{
    StreamExt,
};

use structopt::{
    clap::{
        AppSettings,
    },
    StructOpt,
};

use telegram_bot::{
    Api,
    UpdateKind,
    MessageKind,
    CanReplySendMessage,
};

mod vaccine_reminder;

#[derive(Clone, Debug, StructOpt)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
struct CliArgs {
    /// facebook accounts database
    #[structopt(short = "t", long = "telegram-bot-token")]
    telegram_bot_token: String,
}

#[derive(Debug)]
enum Error {
    TelegramApiStream(telegram_bot::Error),
    TelegramApiSend(telegram_bot::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init_timed();
    let cli_args = CliArgs::from_args();
    log::debug!("cli_args = {:?}", cli_args);

    let api = Api::new(cli_args.telegram_bot_token);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update
            .map_err(Error::TelegramApiStream)?;

        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                println!("{:?}", message);

                // Answer message with "Hi".
                let _message_or_channel_post =
                    api.send(message.text_reply(format!(
                        "Hi, {}! You just wrote '{}'",
                        &message.from.first_name, data
                    )))
                    .await
                    .map_err(Error::TelegramApiSend)?;
            } else {
                println!("other message: {:?}", message);
            }
        } else {
            println!("other update kind: {:?}", update);
        }
    }

    Ok(())
}
