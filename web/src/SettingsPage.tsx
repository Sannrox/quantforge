import { FormEvent, useEffect, useMemo, useState } from "react";
import { api, type Settings } from "./api";

export function SettingsPage() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [provider, setProvider] = useState("fixture");
  const [key, setKey] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api
      .settings()
      .then((row) => {
        setSettings(row);
        setProvider(row.provider);
      })
      .catch((err: Error) => setError(err.message));
  }, []);

  const providers = useMemo(() => settings?.providers ?? ["fixture", "yahoo", "fmp"], [settings]);

  async function save(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setSaved(false);
    try {
      const next = await api.saveSettings({
        provider,
        ...(key ? { fmp_key: key } : {}),
      });
      setSettings(next);
      setKey("");
      setSaved(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not save settings");
    }
  }

  return (
    <section>
      <p className="kicker">Host</p>
      <h1>Settings</h1>
      <p className="note">
        Provider keys stay on this machine. Settings never return the FMP key. Yahoo is unofficial and may fail.
      </p>
      <form className="settings-form" onSubmit={(event) => void save(event)}>
        <label className="field" htmlFor="provider">
          Provider
          <select id="provider" value={provider} onChange={(event) => setProvider(event.target.value)}>
            {providers.map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </label>
        <label className="field" htmlFor="fmp-key">
          FMP API key {settings?.has_fmp_key ? "(saved on host)" : ""}
          <input
            id="fmp-key"
            type="password"
            value={key}
            onChange={(event) => setKey(event.target.value)}
            autoComplete="off"
            placeholder={settings?.has_fmp_key ? "Leave blank to keep" : "Optional"}
          />
        </label>
        <div className="actions">
          <button className="btn" type="submit">
            Save
          </button>
          {settings?.has_fmp_key ? (
            <button
              className="btn btn-quiet"
              type="button"
              onClick={() => {
                setKey("");
                void api
                  .saveSettings({ fmp_key: "" })
                  .then((next) => {
                    setSettings(next);
                    setSaved(true);
                    setError(null);
                  })
                  .catch((err: Error) => setError(err.message));
              }}
            >
              Clear key
            </button>
          ) : null}
        </div>
      </form>
      {saved ? <p className="note" role="status">Saved.</p> : null}
      {error ? (
        <p className="error" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}
