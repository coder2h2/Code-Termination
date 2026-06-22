export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const pathParts = url.pathname.split("/").filter(Boolean);

    // Endpoint: /rooms/:code
    if (pathParts[0] === "rooms" && pathParts.length === 2) {
      const roomCode = pathParts[1];
      const db = env.DB;

      if (!db) {
        return new Response("Configuration Error: DB binding is not set on the worker environment.", { status: 500 });
      }

      const method = request.method.toUpperCase();
      const now = Math.floor(Date.now() / 1000);
      const expirationTime = now - 3600; // 1 hour ago

      if (method === "GET") {
        try {
          const result = await db.prepare(
            "SELECT ip_port FROM rooms WHERE code = ? AND created_at >= ?"
          ).bind(roomCode, expirationTime).first();

          if (!result) {
            return new Response("Room not found", { status: 404 });
          }

          return new Response(result.ip_port, {
            status: 200,
            headers: {
              "Content-Type": "text/plain",
              "Access-Control-Allow-Origin": "*",
            }
          });
        } catch (e) {
          return new Response(`Database Error: ${e.message}`, { status: 500 });
        }
      } else if (method === "PUT") {
        const ipPort = await request.text();
        if (!ipPort || !ipPort.includes(":")) {
          return new Response("Invalid payload: Must provide IP and Port", { status: 400 });
        }

        try {
          // 1. Clean up stale rooms older than 1 hour to keep DB clean
          await db.prepare("DELETE FROM rooms WHERE created_at < ?").bind(expirationTime).run();

          // 2. Insert or update the active room
          await db.prepare(
            "INSERT OR REPLACE INTO rooms (code, ip_port, created_at) VALUES (?, ?, ?)"
          ).bind(roomCode, ipPort.trim(), now).run();

          return new Response("Room registered successfully in DB", {
            status: 200,
            headers: {
              "Access-Control-Allow-Origin": "*",
            }
          });
        } catch (e) {
          return new Response(`Database Error: ${e.message}`, { status: 500 });
        }
      } else if (method === "DELETE") {
        try {
          await db.prepare("DELETE FROM rooms WHERE code = ?").bind(roomCode).run();
          return new Response("Room closed successfully from DB", {
            status: 200,
            headers: {
              "Access-Control-Allow-Origin": "*",
            }
          });
        } catch (e) {
          return new Response(`Database Error: ${e.message}`, { status: 500 });
        }
      }

      // Handle CORS preflight requests
      if (method === "OPTIONS") {
        return new Response(null, {
          status: 204,
          headers: {
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, PUT, DELETE, OPTIONS",
            "Access-Control-Allow-Headers": "Content-Type",
          }
        });
      }

      return new Response("Method not allowed", { status: 405 });
    }

    return new Response("Not Found", { status: 404 });
  }
};
