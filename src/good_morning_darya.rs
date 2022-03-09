
use clap::{
    Parser,
    AppSettings,
};

use telegram_bot::{
    types::{
        UserId,
        GroupId,
        Integer,
    },
};

pub const DEFAULT_USER_ID_STR: &'static str = "621478068"; // Dashasidorova
pub const DEFAULT_GROUP_ID_STR: &'static str = "-222927743"; // Beercan

#[derive(Clone, Debug, Parser)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct CliArgs {
    /// user id to say `good morning` to
    #[clap(long = "good-morning-darya-user-id", default_value = DEFAULT_USER_ID_STR, allow_hyphen_values = true)]
    good_morning_darya_user_id: Integer,

    /// group id to use
    #[clap(long = "good-morning-darya-group-id", default_value = DEFAULT_GROUP_ID_STR, allow_hyphen_values = true)]
    good_morning_darya_group_id: Integer,
}

#[derive(Debug)]
pub enum Error {
    TelegramApiSend(telegram_bot::Error),
}

pub struct GoodMorningDarya {
    user_id: UserId,
    group_id: GroupId,
}

impl GoodMorningDarya {
    pub fn new(cli_args: &CliArgs) -> Result<GoodMorningDarya, Error> {
        Ok(GoodMorningDarya {
            user_id: cli_args.good_morning_darya_user_id.into(),
            group_id: cli_args.good_morning_darya_group_id.into(),
        })
    }
}
