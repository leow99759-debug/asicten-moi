import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CoreEvent } from "./bindings/CoreEvent";

/** Tauri channel carrying every core → UI event (crates/core/src/ipc.rs). */
export const CHANNEL = "core";

export type EventKind = CoreEvent["event"];
export type PayloadOf<K extends EventKind> = Extract<CoreEvent, { event: K }>["payload"];

/** Wrap a typed handler so it only fires for one event kind. */
export function only<K extends EventKind>(kind: K, cb: (payload: PayloadOf<K>) => void) {
  return (ev: CoreEvent): void => {
    if (ev.event === kind) cb(ev.payload as PayloadOf<K>);
  };
}

/** Subscribe to one kind of core event. Returns the unsubscribe function. */
export function on<K extends EventKind>(kind: K, cb: (payload: PayloadOf<K>) => void): Promise<UnlistenFn> {
  const handler = only(kind, cb);
  return listen<CoreEvent>(CHANNEL, (e) => handler(e.payload));
}
