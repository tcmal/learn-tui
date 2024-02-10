use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use edlearn_client::Client;
use log::debug;
use std::{
    fs::File,
    io::Write,
    sync::mpsc::{channel, Receiver, Sender},
};

use super::{ContentIdx, DownloaderRequest, Event};
use crate::event::{Event as CrateEvent, EventBus};

#[derive(Debug, Clone)]
pub struct DownloadReq {
    pub url: String,
    pub orig_filename: String,
    pub dest: Utf8PathBuf,
}

#[derive(Debug, Clone)]
pub enum DownloadState {
    Queued,
    InProgress(f32),
    Completed,
    Errored(String),
}

/// Performs requests it receives from the main thread, and sends the results back.
pub struct Downloader {
    client: Client,
    msg_recv: Receiver<DownloaderRequest>,
    event_send: Sender<CrateEvent>,
}

impl Downloader {
    /// Spawn the store worker on the given event bus, returning a channel to send commands down.
    pub(crate) fn spawn_on(bus: &EventBus, client: Client) -> Sender<DownloaderRequest> {
        let (cmd_send, cmd_recv) = channel();

        bus.spawn("downloader", move |_, event_send| {
            // we don't need running because the receiver will raise an error and we'll exit
            Downloader {
                client,
                msg_recv: cmd_recv,
                event_send,
            }
            .main()
        });

        cmd_send
    }

    fn main(self) {
        while let Ok(msg) = self.msg_recv.recv() {
            debug!("received message: {:?}", msg);
            let DownloaderRequest::DoDownload(r, req) = msg;

            if let Err(e) = match self.do_download(r, req) {
                Ok(_) => self.event_send.send(CrateEvent::Store(Event::DownloadState(
                    r,
                    DownloadState::Completed,
                ))),
                Err(e) => {
                    let e = format!("{:#}", e);
                    self.event_send.send(CrateEvent::Store(Event::DownloadState(
                        r,
                        DownloadState::Errored(e),
                    )))
                }
            } {
                debug!("error sending event: {:?}", e);
                break;
            }
        }

        debug!("shutting down");
    }

    fn do_download(&self, r: ContentIdx, req: DownloadReq) -> Result<(), anyhow::Error> {
        debug!("downloading {req:?} (ref = {r})");

        // make the file
        let mut f = File::create(req.dest.as_std_path())?;

        // start download and find length
        let mut resp = self.client.http().get(req.url).send()?.error_for_status()?;

        // prepare a writer that tracks progress
        let mut writer = ProgressWriter {
            dest: &mut f,
            channel: &self.event_send,
            r,
            size: resp
                .content_length()
                .ok_or_else(|| anyhow!("no content-length header"))?, // TODO: be more graceful about this
            downloaded: 0,
            last_sent: 0.0,
        };

        // do the download
        resp.copy_to(&mut writer)?;

        Ok(())
    }
}

struct ProgressWriter<'a> {
    dest: &'a mut File,
    channel: &'a Sender<CrateEvent>,
    r: ContentIdx,
    downloaded: u64,
    size: u64,
    last_sent: f32,
}

impl<'a> Write for ProgressWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.downloaded += buf.len() as u64;
        let pct = self.downloaded as f32 / self.size as f32;
        if pct - self.last_sent > 0.01 {
            self.channel
                .send(CrateEvent::Store(Event::DownloadState(
                    self.r,
                    DownloadState::InProgress(pct),
                )))
                .unwrap();
            self.last_sent = pct;
        }

        self.dest.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.dest.flush()
    }
}
