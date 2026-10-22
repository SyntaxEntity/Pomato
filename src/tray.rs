use crate::i18n::t;
use std::sync::{Arc, Mutex};

use ksni::menu::*;
use ksni::{Tray, TrayService};

#[derive(Debug, Clone)]
pub enum TrayEvent {
    Show,
    Toggle,
    Reset,
    Skip,
    Quit,
}

struct SharedState {
    title: String,
    icon: String,
    running: bool,
}

struct PomatoTray {
    state: Arc<Mutex<SharedState>>,
    events: flume::Sender<TrayEvent>,
}

impl Tray for PomatoTray {
    fn id(&self) -> String {
        "io.github.pomato.Pomato".into()
    }

    fn icon_name(&self) -> String {
        self.state.lock().unwrap().icon.clone()
    }

    fn title(&self) -> String {
        self.state.lock().unwrap().title.clone()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.events.send(TrayEvent::Show);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let show: MenuItem<Self> = StandardItem {
            label: t("显示 Pomato").into(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.events.send(TrayEvent::Show);
            }),
            ..Default::default()
        }
        .into();

        let running = self.state.lock().unwrap().running;
        let toggle: MenuItem<Self> = StandardItem {
            label: if running { t("暂停") } else { t("开始") }.into(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.events.send(TrayEvent::Toggle);
            }),
            ..Default::default()
        }
        .into();

        let reset: MenuItem<Self> = StandardItem {
            label: t("重置").into(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.events.send(TrayEvent::Reset);
            }),
            ..Default::default()
        }
        .into();

        let skip: MenuItem<Self> = StandardItem {
            label: t("跳过").into(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.events.send(TrayEvent::Skip);
            }),
            ..Default::default()
        }
        .into();

        let quit: MenuItem<Self> = StandardItem {
            label: t("退出").into(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.events.send(TrayEvent::Quit);
            }),
            ..Default::default()
        }
        .into();

        vec![show, toggle, reset, skip, quit]
    }
}

pub struct TrayHandle {
    inner: ksni::Handle<PomatoTray>,
}

impl TrayHandle {
    pub fn set_title(&self, title: impl Into<String>) {
        let title = title.into();
        self.inner.update(|tray| {
            tray.state.lock().unwrap().title = title;
        });
    }

    pub fn set_icon_name(&self, icon: impl Into<String>) {
        let icon = icon.into();
        self.inner.update(|tray| {
            tray.state.lock().unwrap().icon = icon;
        });
    }

    pub fn set_running(&self, running: bool) {
        self.inner.update(|tray| {
            tray.state.lock().unwrap().running = running;
        });
    }
}

pub fn spawn() -> Result<(TrayHandle, flume::Receiver<TrayEvent>), String> {
    let (tx, rx) = flume::unbounded();
    let state = Arc::new(Mutex::new(SharedState {
        title: "Pomato".into(),
        icon: "alarm-symbolic".into(),
        running: false,
    }));

    let tray = PomatoTray {
        state: Arc::clone(&state),
        events: tx,
    };

    let service = TrayService::new(tray);
    let inner = service.handle();
    service.spawn();

    Ok((TrayHandle { inner }, rx))
}
