import "./convert.css";
import "../common.css";
import Convert from "./Convert.svelte";
import { mount } from "svelte";

const app = mount(Convert, {
    target: document.body,
});

export default app;
