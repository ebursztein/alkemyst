import { useId, type InputHTMLAttributes, type ReactNode } from "react";
export function Field({
  label,
  hint,
  id,
  "aria-describedby": describedBy,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label: string; hint?: string }) {
  const generatedId = useId();
  const inputId = id ?? generatedId;
  const description =
    [describedBy, hint ? `${inputId}-hint` : undefined].filter(Boolean).join(" ") || undefined;
  return (
    <div className="field">
      <label htmlFor={inputId}>{label}</label>
      <input {...props} id={inputId} aria-describedby={description} />
      {hint && <small id={`${inputId}-hint`}>{hint}</small>}
    </div>
  );
}
export function FormSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <fieldset className="form-section">
      <legend>{title}</legend>
      <div className="form-grid">{children}</div>
    </fieldset>
  );
}
