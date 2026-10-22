import { Button } from "./button.js";
export function errorMessage(error: unknown): string {
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string")
    return error.message;
  return "The local service could not complete this request.";
}
export function Failure({ error, retry }: { error: unknown; retry?: () => void }) {
  return (
    <div className="notice notice-error" role="alert">
      <p>{errorMessage(error)}</p>
      {retry && (
        <Button color="secondary" onPress={retry}>
          Try again
        </Button>
      )}
    </div>
  );
}
export function Loading({ children = "Loading your library…" }: { children?: string }) {
  return (
    <p className="loading" role="status">
      {children}
    </p>
  );
}
export function Empty({ title, children }: { title: string; children: string }) {
  return (
    <div className="empty">
      <span className="empty-symbol" aria-hidden="true">
        ◇
      </span>
      <h2>{title}</h2>
      <p>{children}</p>
    </div>
  );
}
