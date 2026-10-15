use std::net::UdpSocket;
use std::fmt;
use std:collections::VecDeque;
//Crate imports go here

//This is the function that helps us run NDN packets. 
fn udp() -> std::io::Result<()> {
    {
    // put this here but im unsure abt it -> socket.set_broadcast(true)?;
    }
    Ok(())
}