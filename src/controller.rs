use crate::{PlayerId, Position, SnakeWorld};
use anyhow::{Context, Result};
use bevy::log::*;
use bevy::math::Vec3Swizzles;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Debug, Copy, Clone)]
pub enum MovementCommand {
    NoOps,
    TurnLeft,
    TurnRight,
}
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub username: String,
    pub is_ai: bool,
}
pub trait Controller: 'static + Send + Sync {
    fn initialize(&mut self, player_id: PlayerId) -> Result<PlayerInfo>;
    fn feed_input(&mut self, world: &SnakeWorld) -> Result<()>;
    fn get_output(&mut self) -> Result<MovementCommand>;
}
#[allow(dead_code)]
pub struct StdioController {
    name: String,
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    // read_buf: Vec<u8>
}
macro_rules! writeln {
    ($dst:expr, $($arg:tt)*) => {{
        // info!("writing {:?}", format!($($arg) *));
        std::writeln!($dst, $($arg)*)
    }};
}

fn try_open_file(file: impl AsRef<OsStr>) -> std::io::Result<Child> {
    let file = file.as_ref().to_str().unwrap();
    let args;
    if file.ends_with(".py") {
        args = vec!["python3", file];
    } else {
        args = vec![file.as_ref()];
    }
    let process = Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()?;
    info!("Spawned process {}", process.id());
    Ok(process)
}

impl StdioController {
    pub fn new(file: impl AsRef<OsStr>) -> Result<Self> {
        info!("Loading AI {}", file.as_ref().to_str().unwrap());
        let mut child = try_open_file(file.as_ref())?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        // let mut stderr = child.stderr.take().unwrap();
        // std::thread::spawn(move || loop {
        //     let mut buf = [0u8; 128];
        //     let len = stderr.read(&mut buf).unwrap();
        //     if len == 0 {
        //         break;
        //     }
        //     info!(
        //         "Read from stderr {}",
        //         std::str::from_utf8(&buf[..len]).unwrap()
        //     );
        // });
        Ok(Self {
            name: file.as_ref().to_str().unwrap().to_owned(),
            child,
            stdin,
            stdout,
            // read_buf: vec![]
        })
    }
    fn read_line(&mut self) -> anyhow::Result<String> {
        let mut chunk = [0u8; 1000];
        let len = self.stdout.read(&mut chunk)?;
        let line = String::from_utf8(chunk[..len].to_owned()).unwrap();
        if line.is_empty() {
            anyhow::bail!("Program exited");
        }
        Ok(line)
    }
    pub fn parse_info(&mut self) -> anyhow::Result<PlayerInfo> {
        info!("Parsing player info for AI {}", self.name);
        let line = self.read_line()?;
        let mut spt = line.split(" ");
        let mut info = PlayerInfo {
            username: "".to_string(),
            is_ai: true,
        };
        if spt.next() == Some("username") {
            let username = spt.next().context("Could not find username")?.trim();
            info.username = username.to_owned();
        } else {
            anyhow::bail!("You must begin with username");
        }
        if info.username.is_empty() {
            anyhow::bail!("Could not leave username empty");
        }
        info!("PlayerInfo parsed for AI {}: {:?}", self.name, info);
        Ok(info)
    }
    pub fn parse_action(&mut self) -> anyhow::Result<MovementCommand> {
        let line = self.read_line()?;
        let mut spt = line.split(" ");
        let cmd = spt.next().map(|x| x.trim());
        match cmd {
            Some("turn_left") => Ok(MovementCommand::TurnLeft),
            Some("turn_right") => Ok(MovementCommand::TurnRight),
            Some("straight") => Ok(MovementCommand::NoOps),
            Some(x) => {
                anyhow::bail!("Does not recognize command {:?}", x)
            }
            None => {
                anyhow::bail!("You must not leave an empty line")
            }
        }
    }
}

impl Controller for StdioController {
    fn initialize(&mut self, player_id: PlayerId) -> Result<PlayerInfo> {
        info!("Initializing AI {}", self.name);
        writeln!(self.stdin, "INIT BEGIN")?;
        writeln!(self.stdin, "player_id {}", player_id.0)?;
        writeln!(self.stdin, "INIT END")?;
        std::thread::sleep(std::time::Duration::from_millis(20));
        self.parse_info()
    }

    fn feed_input(&mut self, world: &SnakeWorld) -> Result<()> {
        writeln!(self.stdin, "MAP BEGIN")?;
        for snake in world.snakes.values() {
            write!(self.stdin, "snake {}", snake.player_id.0)?;
            for node in snake.body.values() {
                write!(self.stdin, " {}", Position(node.trans.translation.xy()))?;
            }
            writeln!(self.stdin, "")?;
        }
        for food in &world.foods {
            writeln!(self.stdin, "food {}", food.pos)?;
        }
        writeln!(self.stdin, "MAP END")?;
        Ok(())
    }

    fn get_output(&mut self) -> Result<MovementCommand> {
        writeln!(self.stdin, "REQUEST_ACTION")?;
        self.parse_action()
    }
}
