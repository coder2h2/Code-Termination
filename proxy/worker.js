export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const pathParts = url.pathname.split("/").filter(Boolean);

    // Endpoint: /rooms/:code
    if (pathParts[0] === "rooms" && pathParts.length === 2) {
      const roomCode = pathParts[1];
      const githubToken = env.GITHUB_TOKEN;

      if (!githubToken) {
        return new Response("Configuration Error: GITHUB_TOKEN is not set on the worker environment.", { status: 500 });
      }

      const method = request.method.toUpperCase();

      if (method === "PUT") {
        // Register or update a room
        const ipPort = await request.text();
        if (!ipPort || !ipPort.includes(":")) {
          return new Response("Invalid payload: Must provide IP and Port", { status: 400 });
        }

        // Base64 encode the payload (required by GitHub contents API)
        const encodedIpPort = btoa(ipPort.trim());

        // Check if the file already exists to get its SHA
        const sha = await getSha(roomCode, githubToken);

        const payload = {
          message: `Host room ${roomCode}`,
          content: encodedIpPort,
        };

        if (sha) {
          payload.sha = sha;
        }

        const putUrl = `https://api.github.com/repos/coder2h2/Transmit-Center/contents/${roomCode}.txt`;
        const putResponse = await fetch(putUrl, {
          method: "PUT",
          headers: {
            "Authorization": `token ${githubToken}`,
            "User-Agent": "Code-Termination-Proxy",
            "Content-Type": "application/json"
          },
          body: JSON.stringify(payload)
        });

        if (putResponse.ok) {
          return new Response("Room registered successfully via proxy", { status: 200 });
        } else {
          const errorText = await putResponse.text();
          return new Response(`Failed to register room on GitHub: ${errorText}`, { status: putResponse.status });
        }

      } else if (method === "DELETE") {
        // Close / delete a room
        const sha = await getSha(roomCode, githubToken);
        if (!sha) {
          return new Response("Room not found on GitHub", { status: 404 });
        }

        const payload = {
          message: `Close room ${roomCode}`,
          sha: sha,
        };

        const deleteUrl = `https://api.github.com/repos/coder2h2/Transmit-Center/contents/${roomCode}.txt`;
        const deleteResponse = await fetch(deleteUrl, {
          method: "DELETE",
          headers: {
            "Authorization": `token ${githubToken}`,
            "User-Agent": "Code-Termination-Proxy",
            "Content-Type": "application/json"
          },
          body: JSON.stringify(payload)
        });

        if (deleteResponse.ok) {
          return new Response("Room closed successfully via proxy", { status: 200 });
        } else {
          const errorText = await deleteResponse.text();
          return new Response(`Failed to close room on GitHub: ${errorText}`, { status: deleteResponse.status });
        }
      }

      return new Response("Method not allowed", { status: 405 });
    }

    return new Response("Not Found", { status: 404 });
  }
};

// Helper function to retrieve the SHA of an existing file
async function getSha(roomCode, githubToken) {
  const url = `https://api.github.com/repos/coder2h2/Transmit-Center/contents/${roomCode}.txt`;
  try {
    const response = await fetch(url, {
      headers: {
        "Authorization": `token ${githubToken}`,
        "User-Agent": "Code-Termination-Proxy"
      }
    });
    if (response.status === 200) {
      const data = await response.json();
      return data.sha;
    }
  } catch (e) {
    console.error("Error fetching file SHA:", e);
  }
  return null;
}
