//!

use clap::Parser;
use libcmdproto::proto::{
    abi::{sys_comm, unit_comm, CommandRequest},
    mngr_comm, unit_file, ProstClientStream,
};
use libutils::Error;
use libutils::Result;
use std::net::{SocketAddr, TcpStream};

/// parse program arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of unit
    #[clap(subcommand)]
    subcmd: SubCmd,

    /// Number of times
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

#[derive(Parser, Debug)]
enum SubCmd {
    /// [unit] start the unit
    #[clap(display_order = 1)]
    Start { units: Vec<String> },

    /// [unit] stop the unit
    #[clap(display_order = 2)]
    Stop { units: Vec<String> },

    /// [unit] restart the unit
    #[clap(display_order = 3)]
    Restart { units: Vec<String> },

    /// [unit] status of the unit
    #[clap(display_order = 4)]
    Status { units: Vec<String> },

    /// [manager] list all units
    ListUnits {},

    /// [system] shutdown the system
    Shutdown {},

    /// manager command
    DaemonReload {},

    /// enable one unit file
    Enable { unit_file: Option<String> },

    /// enable one unit file
    Disable { unit_file: Option<String> },

    /// mask one unit file
    Mask { unit_file: Option<String> },

    /// unmask one unit file
    Unmask { unit_file: Option<String> },
}

/// Generate CommandRequest based on parsed args
/// clap Args => protobuf based CommandRequest
fn generate_command_request(args: Args) -> Option<CommandRequest> {
    let command_request = match args.subcmd {
        SubCmd::Start { units } => CommandRequest::new_unitcomm(unit_comm::Action::Start, units),
        SubCmd::Stop { units } => CommandRequest::new_unitcomm(unit_comm::Action::Stop, units),
        SubCmd::Restart { units } => {
            CommandRequest::new_unitcomm(unit_comm::Action::Restart, units)
        }
        SubCmd::Status { units } => CommandRequest::new_unitcomm(unit_comm::Action::Status, units),

        SubCmd::Mask { unit_file } => {
            CommandRequest::new_unitfile(unit_file::Action::Mask, unit_file.unwrap())
        }
        SubCmd::Unmask { unit_file } => {
            CommandRequest::new_unitfile(unit_file::Action::Unmask, unit_file.unwrap())
        }
        SubCmd::Enable { unit_file } => {
            CommandRequest::new_unitfile(unit_file::Action::Enable, unit_file.unwrap())
        }
        SubCmd::Disable { unit_file } => {
            CommandRequest::new_unitfile(unit_file::Action::Disable, unit_file.unwrap())
        }

        SubCmd::Shutdown {} => CommandRequest::new_syscomm(sys_comm::Action::Shutdown),

        SubCmd::ListUnits {} => CommandRequest::new_mngrcomm(mngr_comm::Action::Listunits),
        _ => {
            return None;
        }
    };
    Some(command_request)
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let command_request = match generate_command_request(args) {
        None => {
            println!("This command is currently not supported.");
            return Ok(());
        }
        Some(command_request) => command_request,
    };

    let addrs = [
        SocketAddr::from(([127, 0, 0, 1], 9526)),
        SocketAddr::from(([127, 0, 0, 1], 9527)),
    ];
    let stream = TcpStream::connect(&addrs[..]).unwrap();

    let mut client = ProstClientStream::new(stream);

    let data = client.execute(command_request).unwrap();

    if !data.message.is_empty() {
        println!("{}", data.message);
    }

    Ok(())
}
