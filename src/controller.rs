use crate::{PlayerId, Position, SnakeWorld};
use anyhow::{Context, Result};
use bevy::log::*;
use bevy::math::Vec3Swizzles;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
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
    stdout: BufReader<ChildStdout>,
    // stderr: ChildStderr,
}
fn try_open_file(file: impl AsRef<OsStr>) -> std::io::Result<Child> {
    let file = file.as_ref().to_str().unwrap();
    let args;
    if file.ends_with(".py") {
        args = vec!["python3", file];
    } else {
        args = vec![file.as_ref()];
    }
    Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}
impl StdioController {
    pub fn new(file: impl AsRef<OsStr>) -> Result<Self> {
        info!("Loading AI {}", file.as_ref().to_str().unwrap());
        let mut child = try_open_file(file.as_ref())?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        // let stderr = child.stderr.take().unwrap();
        Ok(Self {
            name: file.as_ref().to_str().unwrap().to_owned(),
            child,
            stdin,
            stdout: BufReader::new(stdout),
            // stderr,
        })
    }
    pub fn parse_info(&mut self) -> anyhow::Result<PlayerInfo> {
        info!("Parsing player info for AI {}", self.name);

        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        let mut spt = line.split(" ");
        let mut info = PlayerInfo {
            username: "".to_string(),
            is_ai: true,
        };
        if spt.next() == Some("username") {
            let username = spt
                .next()
                .map(|x| x.strip_suffix("\n"))
                .flatten()
                .context("Could not find username")?;
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
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        let mut spt = line.split(" ");
        let cmd = spt.next().map(|x| x.strip_suffix("\n")).flatten();
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
        self.stdin.write(b"INIT BEGIN\n")?;
        writeln!(self.stdin, "player_id {}", player_id.0)?;
        self.stdin.write(b"INIT END\n")?;
        self.parse_info()
    }

    fn feed_input(&mut self, world: &SnakeWorld) -> Result<()> {
        self.stdin.write(b"MAP BEGIN\n")?;
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
        self.stdin.write(b"MAP END\n")?;
        Ok(())
    }

    fn get_output(&mut self) -> Result<MovementCommand> {
        writeln!(self.stdin, "REQUEST_ACTION")?;
        self.parse_action()
    }
}
