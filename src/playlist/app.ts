import "./playlist.css";
import "../common.css";
import Playlist from "./Playlist.svelte";
import { mount } from "svelte";

const app = mount(Playlist, {
    target: document.body,
});

export default app;
