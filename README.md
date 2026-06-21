# Code-Termination
A 2D Bevy game based on computers and little bit of comedy
Sprite Makers needed for free please

## Multiplayer

To play online multiplayer, go to the title screen and select **MULTIPLAYER**.

### Hosting a Game
* **With a Personal Access Token (PAT):**
  If you have a GitHub Personal Access Token with write permission to the room registry (`coder2h2/Transmit-Center`), export it as an environment variable (`DLC_PAT` or `GITHUB_TOKEN`) before running the game:
  ```bash
  export DLC_PAT=your_github_token_here
  ```
  The game will generate a **4-digit Room Code** that you can share with your friends.
  
* **Without a PAT (Direct Connect):**
  If you do not have a PAT, you can still host! The game will automatically fall back to direct connection mode. It will display **DIRECT CONNECT** and show your public **IP address and port** (e.g., `192.0.2.1:50505`). Simply share this address with your friends.

### Joining a Game
You do **not** need a PAT to join a game.
* **Using a Room Code:** Enter the 4-digit room code using keyboard digits and press **Enter**.
* **Using Direct Connect:** Type the host's direct IP address and port (including dots `.` and colons `:`) and press **Enter**.
