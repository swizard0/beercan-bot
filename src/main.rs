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
};

mod vaccine_reminder;

#[derive(Clone, Debug, StructOpt)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
struct CliArgs {
    /// facebook accounts database
    #[structopt(short = "t", long = "telegram-bot-token")]
    telegram_bot_token: String,

    #[structopt(flatten)]
    vaccine_reminder: vaccine_reminder::CliArgs,
}

#[derive(Debug)]
enum Error {
    TelegramApiStream(telegram_bot::Error),
    VaccineReminderCreate(vaccine_reminder::Error),
    VaccineReminderProcess(vaccine_reminder::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init_timed();
    let cli_args = CliArgs::from_args();
    log::debug!("cli_args = {:?}", cli_args);

    let api = Api::new(cli_args.telegram_bot_token);

    let vaccine_reminder = vaccine_reminder::VaccineReminder::new(&cli_args.vaccine_reminder)
        .map_err(Error::VaccineReminderCreate)?;

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update
            .map_err(Error::TelegramApiStream)?;

        vaccine_reminder.process(&update, &api).await
            .map_err(Error::VaccineReminderProcess)?;
    }

    Ok(())
}
