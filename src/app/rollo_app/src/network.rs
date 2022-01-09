use std::borrow::{Borrow, BorrowMut};
use std::sync::{Arc, Mutex};
use std::thread::{Thread, ThreadId};
use queue::Queue;
use board_support::{BoardResources, WifiContainer};
use board_support::ServerContainer;
use crate::web_protocol::{Command, ControllerCommand, SchedulerCommand};

pub type ControlCommandBuffer = Arc<Mutex<Queue<ControllerCommand>>>;
pub type SchedulerCommandBuffer = Arc<Mutex<Queue<SchedulerCommand>>>;
type ThreadWakeSignal= Arc<Mutex<Option<ThreadId>>>;
use thiserror::Error;
use bal::server::{Handler};
use bal::networking_types::{Method, Response};
use crate::parse_request;
use bal::server::ServerResource;
use std::thread;
use log::info;
use board_support::wifi::connect_wifi;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't connect to wifi")]
    WifiConnection(board_support::wifi::Error),
    #[error("Couldn't register endpoint")]
    RegisterEndpoint,
    #[error("Couldn't start web server: {0}")]
    ServerCreation(bal::server::Error),
    #[error("Resource busy")]
    ResourceBusy,
    #[error("Resource Not available")]
    ResourceNotAvailable,
    #[error("Can't push command to the buffer")]
    BufferFull,
    #[error("No command to pop from the buffer")]
    BufferEmpty,
}

#[derive(Clone)]
pub struct ThreadSignal <BUF: Clone>{
    wake_signal: Arc<Mutex<Option<Thread>>>,
    command_buffer: Arc<Mutex<Queue<BUF>>>,
}
impl<BUF: Clone> ThreadSignal<BUF> {
    pub fn set_thread(&mut self) -> Result<(), Error>{
        self.wake_signal.lock().map_err(|_|Error::ResourceBusy)?.replace(thread::current());
        Ok(())
    }
    pub fn push_item(&mut self, item: BUF) -> Result<(), Error>{
        self.command_buffer.lock().map_err(|_|Error::ResourceBusy)?.queue(item).map_err(|_|Error::BufferFull)?;
        self.wake_signal.lock().map_err(|_|Error::ResourceBusy)?.as_ref().ok_or(Error::ResourceNotAvailable)?.unpark();
        Ok(())
    }
    pub fn pop_item(&mut self) -> Result<BUF, Error>{
        Ok(self.command_buffer.lock().map_err(|_|Error::ResourceBusy)?.dequeue().ok_or(Error::BufferEmpty)?)
    }
}
#[derive(Clone)]
pub struct ThreadSignals{
    pub blinds_ctrl_signal: ThreadSignal<ControllerCommand>,
    pub scheduler_signal: ThreadSignal<SchedulerCommand>,
}
pub struct Network {
    wifi: WifiContainer,
    server: ServerContainer,
    thread_signals: ThreadSignals,
}

impl Network {
    pub fn new(board: &mut BoardResources) -> Network {
        Network{wifi: board.wifis.get_mut(0).unwrap().take().unwrap(),
            server: board.servers.get_mut(0).unwrap().take().unwrap(),
            thread_signals: ThreadSignals{
            blinds_ctrl_signal: ThreadSignal{ wake_signal: Arc::new(Mutex::new(None)),
                command_buffer: Arc::new(Mutex::new(Queue::with_capacity(2))) },
            scheduler_signal: ThreadSignal{ wake_signal: Arc::new(Mutex::new(None)),
                command_buffer: Arc::new(Mutex::new(Queue::with_capacity(2))) }
            },
        }
    }

    pub fn create_server(&mut self) -> Result<ThreadSignals, Error>{

        #[cfg(target_os = "espidf")]
            {
                println!("Connecting to wifi");
                connect_wifi(self.wifi.borrow_mut()).map_err(|e| Error::WifiConnection(e))?;
            }
        let mut command_handler_thread_signals = self.thread_signals.clone();
        let registry = vec!(
            Handler{
                method: Method::Put,
                uri: "/".to_string(),
                handler: Box::new(move |rq| {handle_command(rq, command_handler_thread_signals.clone()).into()})
            },
        );
        println!("Creating server");
        self.server.create_server(registry).map_err(|e|Error::ServerCreation(e))?;
        Ok(self.thread_signals.clone())
    }


}

fn push_command(command: Command, mut thread_signals: ThreadSignals) ->Result<(), Error>{
    match command {
        Command::Scheduler(sch_command) => {thread_signals.scheduler_signal.push_item(sch_command)?}
        Command::Controller(ctrl_command) => {thread_signals.blinds_ctrl_signal.push_item(ctrl_command)?}
    };
    Ok(())
}

fn handle_command(request: String, mut thread_signals: ThreadSignals) -> Response {
    match parse_request(request) {
        Ok(command) => {
            match push_command(command, thread_signals) {
                Ok(_) => {
                    Response { status: bal::networking_types::Status::Ok, body: format!("{}", "a-ok") }
                }
                Err(error) => { Response { status: bal::networking_types::Status::InternalServerError, body: format!("{}", error) } }
            }
        }
        Err(error) => { Response { status: bal::networking_types::Status::BadRequest, body: format!("{}", error) } }
    }
}
