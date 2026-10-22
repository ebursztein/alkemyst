import { useEffect, useState } from "react";
import { bootstrapConnection } from "./app/connection.js";
import { createServices, type Services } from "./app/services.js";
import { DesktopApp } from "./app/desktop-app.js";
import { Failure, Loading } from "./ui/states.js";
export function Foundation() {
  const [attempt, setAttempt] = useState(0);
  const [state, setState] = useState<{ services?: Services; error?: unknown }>({});
  useEffect(() => {
    let active = true;
    void bootstrapConnection()
      .then((connection) => {
        if (active) setState({ services: createServices(connection) });
      })
      .catch((error: unknown) => {
        if (active) setState({ error });
      });
    return () => {
      active = false;
    };
  }, [attempt]);
  if (state.services) return <DesktopApp services={state.services} />;
  return (
    <main className="bootstrap">
      <div className="bootstrap-mark" aria-hidden="true">
        ⚗
      </div>
      <h1>Alkemyst</h1>
      <p>Your local deck laboratory.</p>
      {state.error ? (
        <Failure
          error={state.error}
          retry={() => {
            setState({});
            setAttempt((a) => a + 1);
          }}
        />
      ) : (
        <Loading>Connecting to your local library…</Loading>
      )}
    </main>
  );
}
