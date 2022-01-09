use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::thread;
use log::{error, info, warn};
use thiserror::Error;
use serde::Deserialize;
use bal::client::ClientResource;
use bal::networking_types::{Method, Request};
use board_support::{BoardResources, ClientContainer};
use time::{Duration, Time};
use url::Position::AfterScheme;
use crate::astronomy::{SunPosition, SunState, SunStateGetter};
use crate::astronomy_api::ip_geolocation;
use crate::BlindsController;
use crate::network::ThreadSignal;
use crate::network;
use crate::web_protocol::{ControllerCommand, SchedulerCommand};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Threading error:{0}")]
    Threading(network::Error),
    #[error("What?")]
    Undefined,
}
enum SchedulerAction {
    OpenBlinds,
    CloseBlinds,
    Idle,
}
pub struct Scheduler {
    client: ClientContainer,
    astronomy_getter: Box<dyn SunStateGetter + Send>,
    thread_signal: ThreadSignal<SchedulerCommand>,
    blinds_ctrl_thread_sig: ThreadSignal<ControllerCommand>,
    running: bool,
    action: SchedulerAction,
}


impl Scheduler {
    pub fn new(board: &mut BoardResources, scheduler_thread_sig: ThreadSignal<SchedulerCommand>, blinds_ctrl_thread_sig: ThreadSignal<ControllerCommand> ) ->Scheduler {
        Scheduler{client: board.clients.get_mut(0).unwrap().take().unwrap(),
            astronomy_getter: Box::new(ip_geolocation::Getter{}),
            thread_signal: scheduler_thread_sig,
            blinds_ctrl_thread_sig: blinds_ctrl_thread_sig,
            running: false,
            action: SchedulerAction::Idle,
        }}

    pub fn start(mut self) {
        info!("Scheduler started!");
       self.run();
    }
    fn process_schedule(&mut self) -> Result<Duration, Error>{
        match self.astronomy_getter.get_sun_state(self.client.borrow_mut()) {
            Ok(sun_s) => {
                match self.action {
                    SchedulerAction::OpenBlinds => {
                        self.blinds_ctrl_thread_sig.push_item(ControllerCommand::Open).map_err(|e|Error::Threading(e));
                    }
                    SchedulerAction::CloseBlinds => {
                        self.blinds_ctrl_thread_sig.push_item(ControllerCommand::Close).map_err(|e|Error::Threading(e));
                    }
                    SchedulerAction::Idle => {}
                }
                let time_to_next_event = sun_s.get_time_to_next_event();

                if time_to_next_event < Duration::hours(2) {
                    self.action = SchedulerAction::Idle;
                    warn!("Next event is not as far as expected. Waiting a little more and trying again");
                    return Ok(time_to_next_event + Duration::minutes(10));
                }
                match sun_s.get_current_sun_pos() {
                    SunPosition::Up => {self.action = SchedulerAction::CloseBlinds;}
                    SunPosition::Down => {self.action = SchedulerAction::OpenBlinds;}
                }
                return Ok(sun_s.get_time_to_next_event() + Duration::minutes(10));

            }
            Err(error) => {
                error!("Could not read astronomy api:{}, Trying again in 1 minute.", error);
                Ok(Duration::minutes(1))
            }
        }
    }

    fn run(&mut self) -> Result<(), Error>{
        // todo this is atrocious, find a better way to schedule stuff
        loop {
            println!("thread woken");
            match self.thread_signal.pop_item() {
                Ok(command) => {
                    match command {
                        SchedulerCommand::Start => {
                            self.running = true;
                            self.action = SchedulerAction::Idle;
                        }
                        SchedulerCommand::Stop => {
                            self.running = false;
                            self.action = SchedulerAction::Idle;
                        }
                    }
                }
                Err(network::Error::BufferEmpty) => {
                    // no item in the buffer means waking up due to timeout
                }
                Err(error) => {
                    error!("Unexpected error reading item from buffer: {}", error);

                }
            }
            if self.running {
                match self.process_schedule() {
                    Ok(sleep) => {
                        let std_sleep = std::time::Duration::new(sleep.whole_seconds() as u64, sleep.subsec_nanoseconds() as u32);
                        info!("Sleeping scheduler for: {}", std_sleep.as_secs());
                        thread::park_timeout( std_sleep);
                    }
                    Err(error) => {
                        error!("Couldn't run scheduler:{}", error);
                    }
                }
            } else {

                error!("Scheduler woken while not running and no new info. going back to sleep");
                thread::park();
            }
        }
    }
}

