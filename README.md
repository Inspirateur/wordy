# wordy
This is the new and improved word cloud discord bot ! 

As opposed to the previous Python version, Wordy starts processing the messages as soon as it joins the server and it's much faster with it ! 
It also automatically continues to process new messages as they are sent.

*Related Blog article: https://teo-orthlieb.github.io/blog/user-word-cloud/*

## How to run it
The discord registration step is the same for all bot that you run yourself.

### Register Discord Application
- go to https://discordapp.com/developers/applications/ create your app
- add a User Bot to it and paste its Token in `token.txt`
- enable `SERVER MEMBERS INTENT` and `MESSAGE CONTENT INTENT` in the bot tab
- invite the bot with `https://discord.com/api/oauth2/authorize?client_id=CLIENT_ID&permissions=0&scope=bot%20applications.commands` replace `CLIENT_ID` with the Client ID of your app

### Run it
- Clone the project wherever you want
- add the `token.txt` file at the root of the project
- Install the necessary tools to compile Rust with https://rustup.rs/
- With a terminal positionned at the root of the project, run `cargo build --release` to build the project.  
*Note: this will take a while because it's the first build you do, subsequent builds will be much faster*
- Run the project with `cargo run --release`

[demo incoming]
