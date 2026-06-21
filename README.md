# Code-Termination
A 2D Bevy game based on computers and little bit of comedy
Sprite Makers needed for free please

## Multiplayer

To play online multiplayer, go to the title screen and select **MULTIPLAYER**.

### Hosting a Game
Select **HOST GAME**. The game will automatically publish to the shared room code registry via a Cloudflare Worker proxy and generate a **4-digit Room Code** that you can share with your friends. No setup or token export is required!

*(Optional: If you want to bypass the proxy and use your own custom GitHub registry token directly, you can export it as an environment variable `export DLC_PAT=your_token` before starting).*

* **Direct Connect Fallback:**
  If GitHub or the proxy is unreachable, or you prefer to connect directly, the hosting game will fall back to displaying `DIRECT (<ip>:<port>)`. You can type in the host's direct public IP and port (e.g. `192.0.2.1:50505`) on the Join screen to connect.

### Joining a Game
1. Select **JOIN GAME**.
2. Type in the **4-digit Room Code** or the host's direct IP address.
3. Press **Enter** to connect.

