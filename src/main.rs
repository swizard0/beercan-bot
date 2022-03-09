#![forbid(unsafe_code)]

use futures::{
    StreamExt,
};

use clap::{
    Parser,
    AppSettings,
};

use telegram_bot::{
    Api,
};

mod vaccine_reminder;
mod delete_recover;

#[derive(Clone, Debug, Parser)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
struct CliArgs {
    /// facebook accounts database
    #[clap(short = 't', long = "telegram-bot-token")]
    telegram_bot_token: String,

    #[clap(flatten)]
    vaccine_reminder: vaccine_reminder::CliArgs,

    #[clap(flatten)]
    delete_recover: delete_recover::CliArgs,
}

#[derive(Debug)]
enum Error {
    TelegramApiStream(telegram_bot::Error),
    VaccineReminderCreate(vaccine_reminder::Error),
    VaccineReminderProcess(vaccine_reminder::Error),
    DeleteRecoverCreate(delete_recover::Error),
    DeleteRecoverProcess(delete_recover::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init_timed();
    let cli_args = CliArgs::parse();
    log::debug!("cli_args = {:?}", cli_args);

    let api = Api::new(cli_args.telegram_bot_token);

    let mut vaccine_reminder = vaccine_reminder::VaccineReminder::new(&cli_args.vaccine_reminder)
        .map_err(Error::VaccineReminderCreate)?;
    let mut delete_recover = delete_recover::DeleteRecover::new(&cli_args.delete_recover)
        .map_err(Error::DeleteRecoverCreate)?;

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update
            .map_err(Error::TelegramApiStream)?;

        vaccine_reminder.process(&update, &api).await
            .map_err(Error::VaccineReminderProcess)?;
        delete_recover.process(&update, &api).await
            .map_err(Error::DeleteRecoverProcess)?;
    }

    Ok(())
}
