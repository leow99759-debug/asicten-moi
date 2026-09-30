import { mount } from "svelte";
import App from "./App.svelte";
import Avatar from "./overlay/Avatar.svelte";
import Hud from "./overlay/Hud.svelte";
import Pill from "./overlay/Pill.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("#app missing");

// Overlay windows load the same bundle with #avatar / #hud / #pill (crates/app/src/overlay.rs).
const view = location.hash.slice(1);
const Root = view === "avatar" ? Avatar : view === "hud" ? Hud : view === "pill" ? Pill : App;
if (Root !== App) document.documentElement.dataset.view = "overlay";

export default mount(Root, { target });
