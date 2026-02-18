import App from "./App.svelte";
import { mount } from "svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error("App root element '#app' was not found");
}

const app = mount(App, {
  target,
});

export default app;
