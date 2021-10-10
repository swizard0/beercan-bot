
use structopt::{
    clap::{
        AppSettings,
    },
    StructOpt,
};

use rand::Rng;

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
    #[structopt(long = "vaccine-reminder-user-id", default_value = DEFAULT_USER_ID_STR)]
    vaccine_reminder_user_id: Integer,

    /// group id to use
    #[structopt(long = "vaccine-reminder-group-id", default_value = DEFAULT_GROUP_ID_STR)]
    vaccine_reminder_group_id: Integer,
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
            user_id: cli_args.vaccine_reminder_user_id.into(),
            group_id: cli_args.vaccine_reminder_group_id.into(),
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
                    } if user_id == &self.user_id && chat_id == &self.group_id && is_question(data) => {
                        let reply_phrase = build_phrase();
                        let _message_or_channel_post = api.send(message.text_reply(reply_phrase)).await
                            .map_err(Error::TelegramApiSend)?;
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

fn build_phrase() -> String {
    let mut rng = rand::thread_rng();

    let mut phrase = String::new();

    if let Some(address) = phrase_address(&mut rng) {
        phrase.push_str(address);
        phrase.push(' ');
    }
    if let Some(name) = phrase_name(&mut rng) {
        phrase.push_str(name);
        phrase.push_str(", ");
    }
    phrase.push_str(phrase_start(&mut rng));
    phrase.push(' ');
    phrase.push_str(phrase_action(&mut rng));
    phrase.push(' ');
    phrase.push_str(phrase_vaccination(&mut rng));
    if let Some(covid) = phrase_covid(&mut rng) {
        phrase.push(' ');
        phrase.push_str(covid);
    }
    if let Some(terminate) = phrase_terminate(&mut rng) {
        phrase.push_str(terminate);
    }
    phrase.push('?');

    phrase
}

fn phrase_address<R>(rng: &mut R) -> Option<&'static str> where R: Rng {
    let variants = &[
        "Уважаемый",
        "Многоуважаемый",
        "Эй,",
        "Слушай,",
        "Послушай,",
        "Извини,",
        "Прошу прощения за беспокойство,",
        "Дружище",
        "Коллега",
    ];

    random_variant_opt(rng, variants)
}

fn phrase_name<R>(rng: &mut R) -> Option<&'static str> where R: Rng {
    let variants = &[
        "Ахмед",
        "Али Баба",
        "Парвиз",
    ];

    random_variant_opt(rng, variants)
}

fn phrase_start<R>(rng: &mut R) -> &'static str where R: Rng {
    let variants = &[
        "а ты уже",
        "ты ещё не",
        "скажи, ты",
        "подскажи, ты",
        "неужели ты",
    ];

    random_variant(rng, variants)
}

fn phrase_action<R>(rng: &mut R) -> &'static str where R: Rng {
    let variants = &[
        "сделал",
        "выполнил",
        "совершил",
        "произвёл",
    ];

    random_variant(rng, variants)
}

fn phrase_vaccination<R>(rng: &mut R) -> &'static str where R: Rng {
    let variants = &[
        "прививку",
        "вакцинацию",
        "укол",
    ];

    random_variant(rng, variants)
}

fn phrase_covid<R>(rng: &mut R) -> Option<&'static str> where R: Rng {
    let variants = &[
        "от коронавируса",
        "от ковида",
        "от covid-19",
        "от понятно какой болезни",
    ];

    random_variant_opt(rng, variants)
}

fn phrase_terminate<R>(rng: &mut R) -> Option<&'static str> where R: Rng {
    let variants = &[
        ", наконец",
        ", в конце концов",
        ", наконец-то",
        ", в конечном итоге",
        ", и, если нет, то по какой причине",
        ", и, если нет, то когда планируешь",
    ];

    random_variant_opt(rng, variants)
}

fn random_variant_opt<R>(rng: &mut R, variants: &[&'static str]) -> Option<&'static str> where R: Rng {
    let variants_count = variants.len();
    let index = rng.gen_range(0 ..= variants_count);
    if index == 0 {
        None
    } else {
        Some(variants[index - 1])
    }
}

fn random_variant<R>(rng: &mut R, variants: &[&'static str]) -> &'static str where R: Rng {
    let variants_count = variants.len();
    let index = rng.gen_range(0 .. variants_count);
    variants[index]
}
