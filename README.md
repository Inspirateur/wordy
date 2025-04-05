# wordy
This is the new and improved word cloud Discord bot! 

As opposed to the previous Python version, Wordy starts processing the messages as soon as it joins a server and is much faster at doing so! 
It also automatically continues to process new messages as they are sent.

*Related blog article: https://teo-orthlieb.github.io/blog/user-word-cloud/*

## Self-hosting guide
The Discord Application creation step is usually the same for all bots that you self-host.

### Create a Discord Application
- Go to https://discord.com/developers/applications/ and create your app
- Add a bot to it and copy its token. You will need it later!
- Enable `PRESENCE INTENT`, `SERVER MEMBERS INTENT` and `MESSAGE CONTENT INTENT` in the bot tab
- Invite the bot with `https://discord.com/oauth2/authorize?client_id=CLIENT_ID&permissions=101376&integration_type=0&scope=bot+applications.commands` and replace `CLIENT_ID` with the Client ID of your app (which you may find under the OAuth2 tab)

**Note:** The OAuth2 link includes following permissions: View Channels, Send Messages, Read Message History (to load messages sent before the bot joined), Attach Files (to send the word cloud image)

### Run it
- Clone the project wherever you want
- Add the `token.txt` file at the root of the project (the root of the project is not inside the src folder, instead the folder with the Cargo files which includes the assets and src folder) and paste the token you copied before into this file (without anything else)
- Install the necessary tools to compile Rust programs by visiting https://rustup.rs/ and copypasting the command into your terminal and following its steps. A standard installation works to build this project.
- With a terminal positioned at the root of the project, run `cargo build --release` to build the project.  
*Note: This will take a while because it's the first build you do, subsequent builds will be much faster.*
- Run the project with `cargo run --release`

### Generate your word cloud
- The bot will load existing messages from the channels it is able to read from.
- Once the messages are loaded (it also works beforehand, which will however not give an extensive result), you can run the bot's slash command `/cloud` to display your word cloud!

#### This is how a word cloud might look like:
![Demonstration word cloud](demo.png)

### Missing Access issue
![image](https://github.com/user-attachments/assets/e2e1b62f-62d8-4df8-9d9a-5ff71bfb1d27)  
Messages like this are caused by private channels. To fix it you need to explicitely add the bot to the private channels of your server, with the "Read Message History" permission.
