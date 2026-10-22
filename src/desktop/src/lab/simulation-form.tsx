import { useServices } from "../app/services.js";
import { useRequestKey } from "../app/request-key.js";
import { Button } from "../ui/button.js";
import { Field, FormSection } from "../ui/field.js";
import { Failure } from "../ui/states.js";
export function SimulationForm({
  versionId,
  supported,
  onCreated,
}: {
  versionId: string;
  supported: boolean;
  onCreated: (id: string) => void;
}) {
  const { api, queryClient } = useServices();
  const key = useRequestKey();
  const mutation = api.useMutation("post", "/v1/simulations", {
    onSuccess: (job) => {
      key.clear();
      onCreated(job.id);
      void queryClient.invalidateQueries();
    },
  });
  return (
    <form
      className="panel simulation-form"
      onSubmit={(event) => {
        event.preventDefault();
        const form = new window.FormData(event.currentTarget);
        const number = (name: string) => Number(form.get(name));
        const seed = String(form.get("seed") ?? "").trim();
        const input = {
          version_id: versionId,
          runs: number("runs"),
          turns: number("turns"),
          on_the_play: form.get("play") === "on",
          seed: seed || undefined,
          scope: "precombat_goldfish" as const,
          goal: {
            equipment: number("equipment"),
            attached: number("attached"),
            attack_ready: number("ready"),
          },
          mulligan: {
            minimum_keep: number("keep"),
            force_mulligans: 0,
            min_lands: number("minLands"),
            max_lands: number("maxLands"),
            required_references: [],
          },
        };
        mutation.mutate({ body: { idempotency_key: key.forBody(input), input } });
      }}
    >
      <div className="panel-heading">
        <h2>Run an experiment</h2>
        <span className="badge">Monte Carlo</span>
      </div>
      <p className="muted">
        Casting, tutor and equip costs are modeled within the supported card interactions.
      </p>
      <FormSection title="Trial settings">
        <Field
          label="Trials"
          name="runs"
          type="number"
          min={1}
          max={1_000_000}
          required
          defaultValue={50_000}
        />
        <Field
          label="Through turn"
          name="turns"
          type="number"
          min={1}
          max={30}
          required
          defaultValue={8}
        />
      </FormSection>
      <FormSection title="Joint battlefield goal">
        <Field
          label="Equipment in play"
          name="equipment"
          type="number"
          min={0}
          max={1000}
          required
          defaultValue={2}
        />
        <Field
          label="Equipment attached"
          name="attached"
          type="number"
          min={0}
          max={1000}
          required
          defaultValue={1}
        />
        <Field
          label="Attached to an attack-ready creature"
          name="ready"
          type="number"
          min={0}
          max={1000}
          required
          defaultValue={1}
        />
      </FormSection>
      <label className="checkbox">
        <input name="play" type="checkbox" defaultChecked />
        On the play (skip the first draw)
      </label>
      <details>
        <summary>London mulligan and replay settings</summary>
        <FormSection title="Keep policy">
          <Field
            label="Minimum keep"
            name="keep"
            type="number"
            min={0}
            max={7}
            required
            defaultValue={4}
          />
          <Field
            label="Minimum lands"
            name="minLands"
            type="number"
            min={0}
            max={7}
            required
            defaultValue={2}
          />
          <Field
            label="Maximum lands"
            name="maxLands"
            type="number"
            min={0}
            max={7}
            required
            defaultValue={5}
          />
        </FormSection>
        <Field
          label="Seed (optional)"
          name="seed"
          pattern="[a-fA-F0-9]{16}"
          placeholder="16 hexadecimal characters"
          hint="Leave blank to generate and save a seed."
        />
      </details>
      {mutation.isError && <Failure error={mutation.error} />}
      <div className="actions">
        <Button type="submit" isLoading={mutation.isPending} isDisabled={!supported}>
          Run simulation
        </Button>
        <small>Saved locally with frozen inputs.</small>
      </div>
    </form>
  );
}
