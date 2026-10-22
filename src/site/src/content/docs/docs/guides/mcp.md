---
title: Connect an agent with MCP
description: Authenticated local Streamable HTTP, connection settings and the operations agents can use.
---

Alkemyst exposes authenticated **Streamable HTTP** on loopback while the tray app
is running. REST, the desktop SDK and MCP invoke the same application operations
and statement engine. MCP has no separate copy of your decks or games.

## Connect

1. Open **Settings → MCP connection** in Alkemyst.
2. Reveal the local credential and copy the current connection configuration.
3. Add the URL and bearer header to an MCP client that supports Streamable HTTP.
   Clients use different configuration formats; adapt the example to your client.
4. Ask the client to discover the available tools before using them.

A generic connection entry looks like this. Use the actual URL and credential
from Settings rather than these placeholders:

```json
{
  "mcpServers": {
    "alkemyst": {
      "url": "http://127.0.0.1:PORT/mcp",
      "headers": { "Authorization": "Bearer LOCAL_CREDENTIAL" }
    }
  }
}
```

Credentials rotate when the app restarts, and the port may change if the preferred
port is occupied. Refresh the client configuration from Settings after a restart.
Closing the window keeps the endpoint alive. Quit stops it.

## Available work

The explicit tool allowlist covers discovery, local card search and resolution,
decks and versions, captured game timelines, observed statistics, exact draw math,
simulation jobs, deck-version creation and notes. Tools derive their schemas from
OpenAPI and return the same typed results and errors as REST.

A useful agent workflow is to inspect a version, check supported interactions,
start a seeded experiment, inspect the returned sample counts and uncertainty,
then propose or create a new deck version. It should not substitute an invented
probability when a model is unsupported or a job fails.

Captured history remains immutable. Agent edits create new deck versions with
agent provenance; they do not overwrite the deck associated with an old game.
There is no arbitrary SQL tool or remote hosted MCP endpoint.

## Privacy and troubleshooting

The bearer credential grants access to the local allowlisted operations. Keep it
private. The app binds loopback and does not support exposing this service on a
public interface or through a website.

Your MCP client may send tool results to an external model provider. Review that
client's data handling before connecting private history. An agent connection is
optional and does not make the agent's processing local.

If a connection fails, verify that Alkemyst is running, copy the current settings,
and check the client's Streamable HTTP and bearer-header support. Do not put the
credential in a URL or public issue.
