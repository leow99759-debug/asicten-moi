// «Дополнения» state (§9): catalog from the core, one-click install/uninstall.
import type { Addon } from "./bindings/Addon";
import type { Pack } from "./bindings/Pack";
import { addonSet, addonsList } from "./commands";
import { inTauri } from "./window";
import { app } from "./app.svelte";
import { ed, load as reloadEditor } from "./editor.svelte";
import { ALL, type Sort } from "./addons";

export const ad = $state({
  list: [] as Addon[],
  busy: {} as Record<string, boolean>,
  tab: "packs",
  q: "",
  cat: ALL as string,
  only: false,
  sort: "popular" as Sort,
  error: "",
});

/** Browser preview: the real catalog straight from the repo. */
async function demo(): Promise<Addon[]> {
  const load = async (files: Record<string, () => Promise<{ default: Pack }>>) =>
    (await Promise.all(Object.values(files).map((f) => f()))).map((m) => m.default);
  const defaults = await load(import.meta.glob<{ default: Pack }>("../../../packs/*.json"));
  const catalog = await load(import.meta.glob<{ default: Pack }>("../../../packs/addons/*.json"));
  return [
    ...defaults.map((pack) => ({ pack, installed: true, default: true })),
    ...catalog.map((pack) => ({ pack, installed: !!pack.popular, default: false })),
  ];
}

export async function load() {
  try {
    ad.list = (inTauri() ? await addonsList() : await demo()) ?? [];
  } catch (e) {
    ad.error = String(e);
  }
}

export async function toggle(id: string, on: boolean) {
  const a = ad.list.find((x) => x.pack.id === id);
  if (!a || a.default || ad.busy[id]) return;
  ad.busy[id] = true;
  ad.error = "";
  a.installed = on; // optimistic, rolled back on error
  try {
    const n = await addonSet(id, on);
    if (n != null) app.commandCount = n;
    if (!ed.dirty) reloadEditor();
  } catch (e) {
    a.installed = !on;
    ad.error = String(e);
  } finally {
    ad.busy[id] = false;
  }
}
