// Small labelled inputs for the sandbox panels.
import { useId, type ReactNode } from 'react';

export interface Option {
  value: string;
  label: string;
}

export function Select(props: { label: string; value: string; options: Option[]; none?: string; onChange: (value: string) => void; testid?: string }) {
  const id = useId();
  return (
    <div className="field">
      <label htmlFor={id}>{props.label}</label>
      <select id={id} className="input" value={props.value} data-testid={props.testid} onChange={(event) => props.onChange(event.target.value)}>
        {props.none !== undefined && <option value="">{props.none}</option>}
        {props.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
}

export function Num(props: { label: string; value: number | undefined; min?: number; max?: number; hint?: string; onChange: (value: number | undefined) => void; testid?: string }) {
  const id = useId();
  return (
    <div className="field">
      <label htmlFor={id}>{props.label}</label>
      <input
        id={id}
        className="input"
        type="number"
        inputMode="numeric"
        min={props.min ?? 0}
        max={props.max}
        value={props.value ?? ''}
        data-testid={props.testid}
        onChange={(event) => {
          const value = event.target.value === '' ? undefined : Math.round(Number(event.target.value));
          props.onChange(value !== undefined && Number.isFinite(value) && value >= (props.min ?? 0) ? value : undefined);
        }}
      />
      {props.hint && <p className="field__hint">{props.hint}</p>}
    </div>
  );
}

export function Text(props: { label: string; value: string; onChange: (value: string) => void; testid?: string }) {
  const id = useId();
  return (
    <div className="field">
      <label htmlFor={id}>{props.label}</label>
      <input id={id} className="input" value={props.value} maxLength={80} data-testid={props.testid} onChange={(event) => props.onChange(event.target.value)} />
    </div>
  );
}

export function Check(props: { label: string; checked: boolean; onChange: (checked: boolean) => void; testid?: string }) {
  return (
    <label className="check">
      <input type="checkbox" checked={props.checked} data-testid={props.testid} onChange={(event) => props.onChange(event.target.checked)} />
      <span>{props.label}</span>
    </label>
  );
}

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="bsec">
      <h3>{title}</h3>
      {children}
    </section>
  );
}
