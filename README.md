<div align=center>
  <h1>tchux - the secure LAN chat app</h1>
  <p>Made to chat covertly over LANs like computer labs</p>
</div>

## What is tchux?
*tchux* (pronounced _chuks_) is a LAN chat app that aims to be secure and easy to set up.

## Installation
*tchux* can be installed via cargo :-
`cargo install tchux`

Alternatively, download a release of your choice from the various releases and run it without Installation

## Usage
*tchux* instances operate in 2 modes, client mode and server mode.

Client mode is used to connect to *tchux* servers on the LAN
whereas server mode initializes a *tchux* server on your machine and then pops a client too so you can join in :P

### Server mode 
`tchux server <port> <password>`

the default port in `8080` and the default password is `IWasTooDumbToSetAPassword`
> Incase it was not obvious, use a password!

> [!NOTE]
> the server command starts an instance of the client too

### Client mode
`tchux client <ip:port> <password>`

If left blank, the password is assumed to be `IWasSoDumbIDidNotSetAPassword`

The client supports commands --prefixed with an `\`. 
Currently only 2 exist :-

| Command | Function         |
| ------- | ---------------- |
| \exit   | exits the client |
| \p      | panic-mode       |

> [!NOTE]
> panic mode exits the client, deletes a few lines from shell history and clears the screen. :)





