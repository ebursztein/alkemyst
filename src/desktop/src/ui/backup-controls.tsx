import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Dialog, Heading, Modal, ModalOverlay } from "react-aria-components";
import { readRestoreStatus, runBackupAction } from "../app/native-backups.js";
import { Button } from "./button.js";
import { Failure, Loading } from "./states.js";

export function BackupControls() {
  const [selected, setSelected] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);
  const [restarting, setRestarting] = useState(false);
  const [working, setWorking] = useState(false);
  const status = useQuery({
    queryKey: ["native-restore-status"],
    queryFn: readRestoreStatus,
    staleTime: Infinity,
  });
  const action = useMutation({
    mutationFn: runBackupAction,
    retry: false,
    onMutate: () => setWorking(true),
    onSettled: (value, error, operation) => {
      // Commit focus-sensitive UI state together after native work finishes.
      setWorking(false);
      if (error) return;
      if (operation === "save" && value) setSaved(value);
      if (operation === "select" && value) setSelected(value);
      if (operation === "cancel") setSelected(null);
      if (operation === "restore") setRestarting(true);
    },
  });
  const busy = working || restarting;
  const choosingRestore = working && action.variables === "select";
  return (
    <section className="panel">
      <h2>Backup and restore</h2>
      <p className="muted">
        Save your decks, versions, games, notes, practice sessions, simulations and card metadata in
        one local file. Backups contain private play history; store and share them carefully.
      </p>
      {status.isPending && <Loading>Reading backup status…</Loading>}
      {status.isError && (
        <Failure
          error={status.error}
          retry={() => {
            void status.refetch();
          }}
        />
      )}
      {status.data?.state === "restored" && (
        <p role="status">Backup restored. Your previous library was kept in a safety backup.</p>
      )}
      {status.data?.state === "failed" && (
        <p role="alert">Restore could not be applied. Your current library was kept.</p>
      )}
      {status.data && status.data.state !== "none" && status.data.safety_backup && (
        <p className="backup-location">
          Safety backup: <code>{status.data.safety_backup}</code>
        </p>
      )}
      {status.data && (
        <>
          <div className="actions">
            <Button color="secondary" isDisabled={busy} onPress={() => action.mutate("save")}>
              Save backup…
            </Button>
            <Button
              color="secondary"
              isDisabled={busy && !choosingRestore}
              aria-disabled={busy || undefined}
              onPress={() => {
                if (!busy) action.mutate("select");
              }}
            >
              Restore backup…
            </Button>
          </div>
          <p className="muted">Choose a new backup file name. Existing files are never replaced.</p>
        </>
      )}
      {saved && (
        <p className="backup-location" role="status">
          Backup saved: <code>{saved}</code>
        </p>
      )}
      {working && !selected && <Loading>Preparing your backup…</Loading>}
      {action.isError && !selected && <Failure error={action.error} />}
      <ModalOverlay
        isOpen={selected !== null}
        isKeyboardDismissDisabled={busy}
        onOpenChange={(open) => {
          if (!open && !busy) action.mutate("cancel");
        }}
        className="backup-overlay"
      >
        <Modal className="backup-modal">
          <Dialog role="alertdialog" aria-describedby="restore-consequences" className="panel">
            <Heading slot="title">Restore from backup?</Heading>
            <p className="backup-location">
              <strong>{selected}</strong>
            </p>
            <p id="restore-consequences">
              Restarting replaces your current library with this backup. Alkemyst keeps a safety
              backup of your current data and restores before tracking or MCP starts.
            </p>
            <p className="muted">
              Card artwork and provider credentials are not included in backups.
            </p>
            <div className="actions">
              <Button
                autoFocus
                color="secondary"
                isDisabled={busy}
                onPress={() => action.mutate("cancel")}
              >
                Keep current library
              </Button>
              <Button color="danger" isDisabled={busy} onPress={() => action.mutate("restore")}>
                Restart and restore
              </Button>
            </div>
            {working && <Loading>Preparing restore…</Loading>}
            {restarting && <p role="status">Restarting Alkemyst to restore your library…</p>}
            {action.isError && <Failure error={action.error} />}
          </Dialog>
        </Modal>
      </ModalOverlay>
    </section>
  );
}
