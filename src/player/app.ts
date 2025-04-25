import "./player.css";
import "../common.css";
import Player from "./Player.svelte";
import { mount } from "svelte";

const app = mount(Player, {
    target: document.body,
});

export default app;
