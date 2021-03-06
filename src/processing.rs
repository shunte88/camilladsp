use audiodevice::*;
use config;
use filters;
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;

pub fn run_processing(
    conf_proc: config::Configuration,
    barrier_proc: Arc<Barrier>,
    tx_pb: mpsc::SyncSender<AudioMessage>,
    rx_cap: mpsc::Receiver<AudioMessage>,
    rx_pipeconf: mpsc::Receiver<(config::ConfigChange, config::Configuration)>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut pipeline = filters::Pipeline::from_config(conf_proc);
        debug!("build filters, waiting to start processing loop");
        barrier_proc.wait();
        loop {
            match rx_cap.recv() {
                Ok(AudioMessage::Audio(mut chunk)) => {
                    trace!("AudioMessage::Audio received");
                    chunk = pipeline.process_chunk(chunk);
                    let msg = AudioMessage::Audio(chunk);
                    tx_pb.send(msg).unwrap();
                }
                Ok(AudioMessage::EndOfStream) => {
                    trace!("AudioMessage::EndOfStream received");
                    let msg = AudioMessage::EndOfStream;
                    tx_pb.send(msg).unwrap();
                    break;
                }
                _ => {}
            }
            if let Ok((diff, new_config)) = rx_pipeconf.try_recv() {
                trace!("Message received on config channel");
                match diff {
                    config::ConfigChange::Pipeline => {
                        debug!("Rebuilding pipeline.");
                        let new_pipeline = filters::Pipeline::from_config(new_config);
                        pipeline = new_pipeline;
                    }
                    config::ConfigChange::FilterParameters { filters, mixers } => {
                        debug!(
                            "Updating parameters of filters: {:?}, mixers: {:?}.",
                            filters, mixers
                        );
                        pipeline.update_parameters(new_config, filters, mixers);
                    }
                    config::ConfigChange::Devices => {
                        let msg = AudioMessage::EndOfStream;
                        tx_pb.send(msg).unwrap();
                        break;
                    }
                    _ => {}
                };
            };
        }
    })
}
