use std::{
    sync::{
        Arc,
    },
    time::{
        Duration,
    },
};

use clap::{
    Parser,
    AppSettings,
};

use chrono::{
    offset::{
        Local,
    },
    Date,
    DateTime,
    NaiveTime,
};

use telegram_bot::{
    types::{
        GroupId,
        Integer,
    },
    Api,
    ParseMode,
    SendMessage,
};

pub const DEFAULT_USERNAME_STR: &'static str = "Dashasidorova";
pub const DEFAULT_GROUP_ID_STR: &'static str = "-222927743"; // Beercan
pub const DEFAULT_REMINDER_TIME_STR: &'static str = "17:00:00";

#[derive(Clone, Debug, Parser)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct CliArgs {
    /// user id to say `good morning` to
    #[clap(long = "good-morning-darya-user-id", default_value = DEFAULT_USERNAME_STR)]
    good_morning_darya_username: String,

    /// group id to use
    #[clap(long = "good-morning-darya-group-id", default_value = DEFAULT_GROUP_ID_STR, allow_hyphen_values = true)]
    good_morning_darya_group_id: Integer,

    /// reminder time
    #[clap(long = "reminder-time", default_value = DEFAULT_REMINDER_TIME_STR)]
    good_morning_darya_reminder_time: String,
}

#[derive(Debug)]
pub enum Error {
    InvalidReminderTime(chrono::ParseError),
    InvalidTodayDatetime {
        date_today: Date<Local>,
        reminder_time: NaiveTime,
    },
    InvalidTomorrowDatetime {
        date_tomorrow: Date<Local>,
        reminder_time: NaiveTime,
    },
    TelegramApiSend(telegram_bot::Error),
}

pub struct GoodMorningDarya {
    _reminder_task: tokio::task::JoinHandle<()>,
}

impl GoodMorningDarya {
    pub fn new(api: Arc<Api>, cli_args: &CliArgs) -> Result<GoodMorningDarya, Error> {
        let reminder_time = parse_reminder_time(&cli_args.good_morning_darya_reminder_time)?;
        let username = cli_args.good_morning_darya_username.clone();
        let group_id = cli_args.good_morning_darya_group_id.into();
        let reminder_task = tokio::spawn(reminder_loop(api, reminder_time, username, group_id));
        Ok(GoodMorningDarya {
            _reminder_task: reminder_task,
        })
    }
}

async fn reminder_loop(
    api: Arc<Api>,
    reminder_time: NaiveTime,
    username: String,
    group_id: GroupId,
)
{
    log::debug!("starting reminder loop on {:?} for {:?} in {:?}", reminder_time, username, group_id);
    if let Err(error) = reminder_loop_run(api, reminder_time, username, group_id).await {
        log::error!("reminder loop terminated with error: {:?}", error);
    }
}

async fn reminder_loop_run(
    api: Arc<Api>,
    reminder_time: NaiveTime,
    username: String,
    group_id: GroupId,
)
    -> Result<(), Error>
{
    loop {
        let datetime_now = Local::now();
        let datetime_reminder = nearest_reminder_datetime_by(datetime_now, reminder_time)?;
        let timeout_ms = next_timeout(datetime_now, datetime_reminder);
        tokio::time::sleep(Duration::from_millis(timeout_ms)).await;

        let mut good_morning_message =
            SendMessage::new(&group_id, format!("Доброе утро, @{} !", username));
        good_morning_message.parse_mode(ParseMode::Markdown);
        api.send(good_morning_message).await
            .map_err(Error::TelegramApiSend)?;
    }
}

fn parse_reminder_time(string: &str) -> Result<NaiveTime, Error> {
    NaiveTime::parse_from_str(string, "%H:%M:%S")
        .map_err(Error::InvalidReminderTime)
}

fn nearest_reminder_datetime_by(datetime_now: DateTime<Local>, reminder_time: NaiveTime) -> Result<DateTime<Local>, Error> {
    let date_today = datetime_now.date();
    let datetime_today = date_today
        .and_time(reminder_time)
        .ok_or_else(|| Error::InvalidTodayDatetime { date_today, reminder_time, })?;
    if datetime_now < datetime_today {
        Ok(datetime_today)
    } else {
        let date_tomorrow = date_today.succ();
        let datetime_tomorrow = date_tomorrow
            .and_time(reminder_time)
            .ok_or_else(|| Error::InvalidTomorrowDatetime { date_tomorrow, reminder_time, })?;
        Ok(datetime_tomorrow)
    }
}

fn next_timeout(datetime_now: DateTime<Local>, datetime_reminder: DateTime<Local>) -> u64 {
    let reminder_datetime_millis = datetime_reminder.timestamp_millis();
    let datetime_now_millis = datetime_now.timestamp_millis();
    (reminder_datetime_millis - datetime_now_millis) as u64
}

#[cfg(test)]
mod tests {
    use chrono::{
        offset::{
            Local,
        },
        TimeZone,
        NaiveTime,
    };

    use super::{
        next_timeout,
        parse_reminder_time,
        nearest_reminder_datetime_by,
    };

    #[test]
    fn parse_reminder_time_17_00() {
        assert_eq!(
            parse_reminder_time("17:00:00").unwrap(),
            NaiveTime::from_hms(17, 0, 0),
        );
    }

    #[test]
    fn nearest_reminder_date_today() {
        assert_eq!(
            nearest_reminder_datetime_by(
                Local.ymd(2022, 3, 9)
                    .and_hms(11, 47, 24),
                NaiveTime::from_hms(17, 0, 0),
            ).unwrap(),
            Local.ymd(2022, 3, 9)
                .and_hms(17, 0, 0),
        );
    }

    #[test]
    fn nearest_reminder_date_tomorrow() {
        assert_eq!(
            nearest_reminder_datetime_by(
                Local.ymd(2022, 3, 9)
                    .and_hms(19, 47, 24),
                NaiveTime::from_hms(17, 0, 0),
            ).unwrap(),
            Local.ymd(2022, 3, 10)
                .and_hms(17, 0, 0),
        );
    }

    #[test]
    fn nearest_reminder_date_now() {
        assert_eq!(
            nearest_reminder_datetime_by(
                Local.ymd(2022, 3, 9)
                    .and_hms(17, 0, 0),
                NaiveTime::from_hms(17, 0, 0),
            ).unwrap(),
            Local.ymd(2022, 3, 10)
                .and_hms(17, 0, 0),
        );
    }

    #[test]
    fn next_timeout_1000() {
        let datetime_now = Local.ymd(2022, 3, 9)
            .and_hms(16, 59, 59);
        let datetime_reminder = nearest_reminder_datetime_by(
            datetime_now,
            NaiveTime::from_hms(17, 0, 0),
        ).unwrap();
        assert_eq!(
            next_timeout(datetime_now, datetime_reminder),
            1000,
        );
    }

}
