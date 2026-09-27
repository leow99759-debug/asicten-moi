import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CoreEvent } from "./bindings/CoreEvent";

/** Tauri channel carrying every core → UI event (crates/core/src/ipc.rs). */
export const CHANNEL = "core";

export type EventKind = CoreEvent["event"];
/** Payload type of an event kind; `undefined` for payload-less events like `confirm_closed`. */
export type PayloadOf<K extends EventKind> =
  Extract<CoreEvent, { event: K }> extends { payload: infer P } ? P : undefined;

/** Wrap a typed handler so it only fires for one event kind. */
export function only<K extends EventKind>(kind: K, cb: (payload: PayloadOf<K>) => void) {
  return (ev: CoreEvent): void => {
    if (ev.event === kind) cb(("payload" in ev ? ev.payload : undefined) as PayloadOf<K>);
  };
}

/** Subscribe to one kind of core event. Returns the unsubscribe function. */
export function on<K extends EventKind>(kind: K, cb: (payload: PayloadOf<K>) => void): Promise<UnlistenFn> {
  const handler = only(kind, cb);
  return listen<CoreEvent>(CHANNEL, (e) => handler(e.payload));
}
