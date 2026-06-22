export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const pathParts = url.pathname.split("/").filter(Boolean);

    // Endpoint: /rooms/:code
    if (pathParts[0] === "rooms" && pathParts.length === 2) {
      const roomCode = pathParts[1];
      const roomsKv = env.ROOMS_KV;

      if (!roomsKv) {
        return new Response("Configuration Error: ROOMS_KV binding is not set on the worker environment.", { status: 500 });
      }

      const method = request.method.toUpperCase();

      if (method === "GET") {
        const ipPort = await roomsKv.get(roomCode);
        if (!ipPort) {
          return new Response("Room not found", { status: 404 });
        }
        return new Response(ipPort, {
          status: 200,
          headers: {
            "Content-Type": "text/plain",
            "Access-Control-Allow-Origin": "*",
          }
        });
      } else if (method === "PUT") {
        const ipPort = await request.text();
        if (!ipPort || !ipPort.includes(":")) {
          return new Response("Invalid payload: Must provide IP and Port", { status: 400 });
        }

        // Store the room mapping with a 1-hour expiration time (TTL) to automatically clean up stale rooms
        await roomsKv.put(roomCode, ipPort.trim(), { expirationTtl: 3600 });
        return new Response("Room registered successfully in KV", {
          status: 200,
          headers: {
            "Access-Control-Allow-Origin": "*",
          }
        });
      } else if (method === "DELETE") {
        await roomsKv.delete(roomCode);
        return new Response("Room closed successfully from KV", {
          status: 200,
          headers: {
            "Access-Control-Allow-Origin": "*",
          }
        });
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
