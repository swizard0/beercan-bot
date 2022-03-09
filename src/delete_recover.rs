use std::{
    time::{
        Duration,
    },
    collections::{
        VecDeque,
    },
};

use futures::{
    channel::{
        mpsc,
    },
    select,
    SinkExt,
    FutureExt,
    StreamExt,
};

use tokio::{
    time::{
        sleep,
    },
};

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
    Api,
    User,
    Group,
    Update,
    Message,
    ParseMode,
    UpdateKind,
    MessageChat,
    SendMessage,
    MessageKind,
    CanDeleteMessage,
    CanForwardMessage,
};

pub const DEFAULT_USER_ID_STR: &'static str = "337229462"; // Parviz Sadesi
pub const DEFAULT_GROUP_ID_STR: &'static str = "-222927743"; // Beercan
pub const DEFAULT_FORWARD_GROUP_ID_STR: &'static str = "-756453207"; // beercan delete monitor
pub const DEFAULT_WINDOW_SIZE_STR: &'static str = "32";
pub const DEFAULT_CHECK_TIMEOUT_S_STR: &'static str = "60";

#[derive(Clone, Debug, Parser)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct CliArgs {
    /// user id to remind about vaccination
    #[clap(long = "delete-recover-user-id", default_value = DEFAULT_USER_ID_STR, allow_hyphen_values = true)]
    delete_recover_user_id: Integer,

    /// group id to use
    #[clap(long = "delete-recover-group-id", default_value = DEFAULT_GROUP_ID_STR, allow_hyphen_values = true)]
    delete_recover_group_id: Integer,

    /// group id to forward messages to (delete monitor)
    #[clap(long = "delete-recover-forward-group-id", default_value = DEFAULT_FORWARD_GROUP_ID_STR, allow_hyphen_values = true)]
    delete_recover_forward_group_id: Integer,

    /// messages window size to monitor
    #[clap(long = "delete-recover-window-size", default_value = DEFAULT_WINDOW_SIZE_STR)]
    delete_recover_window_size: usize,

    /// check timeout before trying to forward messages (in seconds)
    #[clap(long = "delete-recover-check-timeout-s", default_value = DEFAULT_CHECK_TIMEOUT_S_STR)]
    delete_recover_check_timeout_s: u64,
}

#[derive(Debug)]
pub enum Error {
    MonitorTaskIsGone,
}

pub struct DeleteRecover {
    user_id: UserId,
    group_id: GroupId,
    forward_group_id: GroupId,
    window_size: usize,
    check_timeout_s: u64,
    maybe_monitor_tx: Option<mpsc::Sender<Message>>,
}

impl DeleteRecover {
    pub fn new(cli_args: &CliArgs) -> Result<DeleteRecover, Error> {
        Ok(DeleteRecover {
            user_id: cli_args.delete_recover_user_id.into(),
            group_id: cli_args.delete_recover_group_id.into(),
            forward_group_id: cli_args.delete_recover_forward_group_id.into(),
            window_size: cli_args.delete_recover_window_size,
            check_timeout_s: cli_args.delete_recover_check_timeout_s,
            maybe_monitor_tx: None,
        })
    }

    pub async fn process(&mut self, update: &Update, api: &Api) -> Result<(), Error> {
        match &update.kind {
            UpdateKind::Message(message) =>
                match message {
                    Message {
                        from: User { id: user_id, .. },
                        chat: MessageChat::Group(Group { id: chat_id, .. }),
                        ..
                    } if user_id == &self.user_id && chat_id == &self.group_id => {
                        let monitor_tx = if let Some(monitor_tx) = &mut self.maybe_monitor_tx {
                            monitor_tx
                        } else {
                            let (monitor_tx, monitor_rx) = mpsc::channel(0);
                            tokio::spawn(run_monitor(
                                api.clone(),
                                monitor_rx,
                                self.group_id,
                                self.forward_group_id,
                                self.window_size,
                                self.check_timeout_s,
                            ));

                            log::info!("monitor task has spawned");
                            self.maybe_monitor_tx
                                .get_or_insert(monitor_tx)
                        };
                        monitor_tx.send(message.clone()).await
                            .map_err(|_send_error| Error::MonitorTaskIsGone)?;
                    },
                    _other_message =>
                        (),
                },
            _other_update =>
                (),
        }
        Ok(())
    }
}

async fn run_monitor(
    api: Api,
    monitor_rx: mpsc::Receiver<Message>,
    group_id: GroupId,
    forward_group_id: GroupId,
    window_size: usize,
    check_timeout_s: u64,
)
{
    let mut fused_monitor_rx = monitor_rx.fuse();
    let mut current_timeout = None;
    let mut window = VecDeque::with_capacity(window_size);
    let mut window_backup = Vec::with_capacity(window_size);

    loop {
        if current_timeout.is_none() {
            current_timeout = Some(Box::pin(sleep(Duration::from_secs(check_timeout_s)).fuse()));
        }

        enum Event<M> {
            Message(M),
            MonitorTimeout,
        }

        let event = if let Some(mut sleep_future) = current_timeout.as_mut() {
            select! {
                result = fused_monitor_rx.next() =>
                    Event::Message(result),
                () = sleep_future =>
                    Event::MonitorTimeout,
            }
        } else {
            Event::Message(fused_monitor_rx.next().await)
        };

        match event {

            Event::Message(None) => {
                log::info!("monitor rx channel dropped: terminating");
                break;
            },

            Event::Message(Some(message)) => {
                log::debug!("remembering message: [ {:?} ]", message);
                while window.len() >= window_size {
                    window.pop_front();
                }
                window.push_back(message);
            },

            Event::MonitorTimeout => {
                current_timeout = None;
                for message in window.drain(..) {
                    let mut forward_message = message.forward(&forward_group_id);
                    forward_message.disable_notification();
                    match api.send(forward_message).await {
                        Ok(message_or_channel_post) => {
                            api.send(message_or_channel_post.delete()).await.ok();
                            window_backup.push(message);
                        },
                        Err(error) if format!("{}", error).contains("message to forward not found") => {
                            log::debug!("detected deleted message: {:?}", message);
                            let who = if let Some(username) = &message.from.username {
                                format!("@{}", username)
                            } else {
                                message.from.first_name.to_string()
                            };
                            let source = match &message.kind {
                                MessageKind::Text { data, .. } |
                                MessageKind::Document { caption: Some(data), .. } |
                                MessageKind::Photo { caption: Some(data), .. } |
                                MessageKind::Video { caption: Some(data), .. } =>
                                    format!("\n```\n{}\n```\n", data),
                                _ =>
                                    String::new(),
                            };
                            let mut monitor_notify_message =
                                SendMessage::new(&group_id, format!("{} , вот злодей, удалил сообщение!{}", who, source));
                            monitor_notify_message.parse_mode(ParseMode::Markdown);
                            api.send(monitor_notify_message).await.ok();
                        },
                        Err(error) => {
                            log::error!("failed to forward: {:?}", error);
                            break;
                        },
                    }
                }
                window.extend(window_backup.drain(..));
            },

        }


    }
}
