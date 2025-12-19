use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

#[derive(Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_reason: String,
}

#[derive(Serialize, Deserialize)]
pub enum CommandToQinit {
    GetLoginCredentials,
    StopListening,
    TriggerSplash(Splash),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AnswerFromQinit {
    Login(Option<LoginForm>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub enum Splash {
    PowerOff,
    Reboot,
    Sleep,
}

pub fn bind(path: &str) -> Result<UnixListener> {
    info!("Binding or creating UNIX socket at path '{}'", &path);
    if fs::exists(&path)? {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove existing socket at path '{}'", &path))?;
    }
    let unix_listener = UnixListener::bind(&path)
        .with_context(|| format!("Could not bind to UNIX socket at path '{}'", &path))?;

    Ok(unix_listener)
}

pub fn read(unix_listener: &UnixListener) -> Result<Vec<u8>> {
    info!("Listening on UNIX socket at {:?}", &unix_listener);
    let (unix_stream, _socket_address) = unix_listener
        .accept()
        .with_context(|| "Failed to accept connection on UNIX socket")?;
    let message_bytes = read_from_stream(&unix_stream)?;

    Ok(message_bytes)
}

pub fn read_from_stream(mut stream: &UnixStream) -> Result<Vec<u8>> {
    let mut message_bytes = Vec::new();
    stream
        .read_to_end(&mut message_bytes)
        .with_context(|| "Failed to read from UNIX socket")?;

    Ok(message_bytes)
}

pub fn write(path: &str, contents: &Vec<u8>) -> Result<UnixStream> {
    info!("Writing {:?} to UNIX socket at path '{}'", &contents, &path);
    let mut stream = connect(&path)?;
    stream.write_all(&contents)?;
    stream.shutdown(std::net::Shutdown::Write)?;

    Ok(stream)
}

pub fn write_and_read(path: &str, contents: &Vec<u8>) -> Result<Vec<u8>> {
    Ok(read_from_stream(&write(&path, &contents)?)?)
}

pub fn connect(path: &str) -> Result<UnixStream> {
    let unix_stream = UnixStream::connect(&path)
        .with_context(|| format!("Failed to connect to socket at path '{}'", &path))?;

    Ok(unix_stream)
}
